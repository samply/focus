use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::NaiveDate;
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

use super::ast::{Exists, Expression, Predicate, Step};
use super::mapping::{Field, FieldType, Mapping, Scope};
use super::Error;
use crate::cql::escape;

fn rejected<T>(message: String) -> Result<T, Error> {
    Err(Error::Rejected(message))
}

fn scope<'a>(mapping: &'a Mapping, key: &str) -> Result<&'a Scope, Error> {
    match mapping.scopes.get(key) {
        Some(scope) => Ok(scope),
        None => rejected(format!("unknown scope \"{key}\"")),
    }
}

fn date(value: &Option<String>, absent: &str) -> Result<String, Error> {
    match value {
        None => Ok(absent.into()),
        Some(d) if NaiveDate::parse_from_str(d, "%Y-%m-%d").is_ok() => Ok(format!("@{d}")),
        Some(d) => rejected(format!("invalid date \"{d}\"")),
    }
}

fn number(value: &Option<f64>, absent: &str) -> String {
    value.map_or(absent.into(), |n| n.to_string())
}

/// `place` names where the fields come from, for error messages.
fn predicate(
    predicate: &Predicate,
    fields: &BTreeMap<String, Field>,
    place: &str,
    this: &str,
) -> Result<String, Error> {
    let (name, type_, value, min, max) = match predicate {
        Predicate::And { operands } | Predicate::Or { operands } => {
            if operands.is_empty() {
                return rejected("empty and/or".into());
            }
            let parts = operands
                .iter()
                .map(|o| predicate_in_parens(o, fields, place, this))
                .collect::<Result<Vec<_>, _>>()?;
            let op = match predicate {
                Predicate::And { .. } => " and ",
                _ => " or ",
            };
            return Ok(parts.join(op));
        }
        Predicate::Values { field, values } => {
            let list = values
                .iter()
                .map(|v| format!("'{}'", escape(v)))
                .collect::<Vec<_>>()
                .join(", ");
            (
                field,
                FieldType::Values,
                format!("{{{list}}}"),
                String::new(),
                String::new(),
            )
        }
        Predicate::Date { field, min, max } => (
            field,
            FieldType::Date,
            String::new(),
            date(min, "minimum Date")?,
            date(max, "maximum Date")?,
        ),
        Predicate::Range { field, min, max } => (
            field,
            FieldType::Range,
            String::new(),
            number(min, "minimum Decimal"),
            number(max, "maximum Decimal"),
        ),
    };
    let Some(field) = fields.get(name) else {
        return rejected(format!("unknown field \"{name}\" in {place}"));
    };
    if field.type_ != type_ {
        return rejected(format!(
            "field \"{name}\" in {place} is not of type {type_:?}"
        ));
    }
    Ok(field
        .cql
        .replace("{{this}}", this)
        .replace("{{value}}", &value)
        .replace("{{min}}", &min)
        .replace("{{max}}", &max))
}

fn predicate_in_parens(
    p: &Predicate,
    fields: &BTreeMap<String, Field>,
    place: &str,
    this: &str,
) -> Result<String, Error> {
    Ok(format!("({})", predicate(p, fields, place, this)?))
}

/// The scope's filter and the chip's criteria on `alias`, if there are any.
fn scoped_predicates(exists: &Exists, scope: &Scope, alias: &str) -> Result<Vec<String>, Error> {
    let place = format!("scope \"{}\"", exists.scope);
    let mut predicates: Vec<String> = scope
        .filter
        .iter()
        .map(|f| format!("({})", f.replace("{{this}}", alias)))
        .collect();
    if let Some(p) = &exists.operand {
        predicates.push(predicate_in_parens(p, &scope.fields, &place, alias)?);
    }
    Ok(predicates)
}

/// `[Type] alias where ...`, or just `[Type]` without predicates.
fn source(resource_type: &str, alias: &str, predicates: &[String]) -> String {
    if predicates.is_empty() {
        format!("[{resource_type}]")
    } else {
        format!(
            "[{resource_type}] {alias} where {}",
            predicates.join("\n  and ")
        )
    }
}

fn exists(exists: &Exists, mapping: &Mapping) -> Result<String, Error> {
    let scope = scope(mapping, &exists.scope)?;
    let predicates = scoped_predicates(exists, scope, "Q")?;
    Ok(format!(
        "exists({})",
        source(&scope.resource_type, "Q", &predicates)
    ))
}

fn temporal_date(scope: &Scope, key: &str, alias: &str) -> Result<String, Error> {
    match &scope.temporal_date {
        Some(d) => Ok(format!("({})", d.replace("{{this}}", alias))),
        None => rejected(format!(
            "scope \"{key}\" can't be used in a temporal relation"
        )),
    }
}

fn as_exists(e: &Expression) -> Result<&Exists, Error> {
    match e {
        Expression::Exists(x) => Ok(x),
        _ => rejected("temporal relations only take exists nodes".into()),
    }
}

/// Each step is nested inside the one before it, so a chain "A then B then C"
/// compares against one B rather than two.
fn temporal(first: &Expression, steps: &[Step], mapping: &Mapping) -> Result<String, Error> {
    if steps.is_empty() {
        return rejected("temporal relation without steps".into());
    }
    let mut chain = vec![(as_exists(first)?, None)];
    for step in steps {
        chain.push(match step {
            Step::Then { operand } => (as_exists(operand)?, None),
            Step::WithinDays { days, operand } => (as_exists(operand)?, Some(format!("<= {days}"))),
            Step::AtLeastDays { days, operand } => {
                (as_exists(operand)?, Some(format!(">= {days}")))
            }
        });
    }

    let mut nested = None;
    for i in (0..chain.len()).rev() {
        let (exists, limit) = &chain[i];
        let alias = format!("T{i}");
        let scope = scope(mapping, &exists.scope)?;
        let mut predicates = scoped_predicates(exists, scope, &alias)?;
        if i > 0 {
            let before = chain[i - 1].0;
            let earlier = temporal_date(
                self::scope(mapping, &before.scope)?,
                &before.scope,
                &format!("T{}", i - 1),
            )?;
            let later = temporal_date(scope, &exists.scope, &alias)?;
            predicates.push(format!("{later} > {earlier}"));
            if let Some(limit) = limit {
                predicates.push(format!("days between {earlier} and {later} {limit}"));
            }
        }
        predicates.extend(nested.take());
        nested = Some(format!(
            "exists({})",
            source(&scope.resource_type, &alias, &predicates)
        ));
    }
    Ok(nested.unwrap_or_default())
}

fn expression(e: &Expression, mapping: &Mapping) -> Result<String, Error> {
    Ok(match e {
        Expression::And { operands } | Expression::Or { operands } => {
            if operands.is_empty() {
                return rejected("empty and/or".into());
            }
            let parts = operands
                .iter()
                .map(|o| Ok(format!("({})", expression(o, mapping)?)))
                .collect::<Result<Vec<_>, Error>>()?;
            let op = match e {
                Expression::And { .. } => "\n  and ",
                _ => "\n  or ",
            };
            parts.join(op)
        }
        Expression::Not { operand } => format!("not ({})", expression(operand, mapping)?),
        Expression::MatchAll => "true".into(),
        Expression::Subject { operand } => {
            predicate(operand, &mapping.fields, "unscoped fields", "Patient")?
        }
        Expression::Exists(x) => exists(x, mapping)?,
        Expression::Temporal { first, steps } => temporal(first, steps, mapping)?,
    })
}

/// The query's chips on the `specimen` scope, so specimen counts only include
/// specimens a chip asked for. Negated parts are left out.
fn specimen_criteria<'a>(e: &'a Expression, out: &mut Vec<&'a Exists>) {
    match e {
        Expression::And { operands } | Expression::Or { operands } => {
            operands.iter().for_each(|o| specimen_criteria(o, out))
        }
        Expression::Exists(x) if x.scope == "specimen" => out.push(x),
        Expression::Temporal { first, steps } => {
            specimen_criteria(first, out);
            for step in steps {
                let (Step::Then { operand }
                | Step::WithinDays { operand, .. }
                | Step::AtLeastDays { operand, .. }) = step;
                specimen_criteria(operand, out);
            }
        }
        _ => {}
    }
}

/// What a count or stratifier counts: patients of the cohort, or resources of
/// cohort patients, narrowed by its filter and, for specimens, by the query's
/// specimen criteria.
fn members_define(
    name: &str,
    resource_type: &str,
    filter: &Option<String>,
    e: &Expression,
    mapping: &Mapping,
) -> Result<String, Error> {
    let this = if resource_type == "Patient" {
        "Patient"
    } else {
        "M"
    };
    let mut predicates: Vec<String> = filter
        .iter()
        .map(|f| format!("({})", f.replace("{{this}}", this)))
        .collect();
    if resource_type == "Patient" {
        predicates.insert(0, "Cohort".into());
        return Ok(format!(
            "define \"{name}\":\n  {}",
            predicates.join(" and ")
        ));
    }
    if resource_type == "Specimen" {
        let mut criteria = Vec::new();
        specimen_criteria(e, &mut criteria);
        let mut parts = Vec::new();
        for x in criteria {
            let p = scoped_predicates(x, scope(mapping, &x.scope)?, "M")?;
            parts.push(if p.is_empty() {
                "true".into()
            } else {
                p.join(" and ")
            });
        }
        if !parts.is_empty() {
            predicates.push(format!("(({}))", parts.join(") or (")));
        }
    }
    Ok(format!(
        "define \"{name}\":\n  if Cohort then {} else {{}} as List<FHIR.{resource_type}>",
        source(resource_type, "M", &predicates)
    ))
}

fn group(name: &str, resource_type: &str, members: String, stratifier: Option<&str>) -> Value {
    let mut group = json!({
        "code": {"text": name},
        "population": [{
            "code": {"coding": [{"system": "http://terminology.hl7.org/CodeSystem/measure-population", "code": "initial-population"}]},
            "criteria": {"language": "text/cql-identifier", "expression": members},
        }],
    });
    if let Some(stratifier) = stratifier {
        group["stratifier"] = json!([{
            "code": {"text": name},
            "criteria": {"language": "text/cql-identifier", "expression": stratifier},
        }]);
    }
    if resource_type != "Patient" {
        group["extension"] = json!([{
            "url": "http://hl7.org/fhir/us/cqfmeasures/StructureDefinition/cqfm-populationBasis",
            "valueCode": resource_type,
        }]);
    }
    group
}

/// Build the Library and Measure that evaluate `e` against the mapping.
pub fn generate(e: &Expression, mapping: &Mapping) -> Result<(Value, Value), Error> {
    // Every count and stratifier gets its own group, even where they count the
    // same members.
    let mut defines = vec![format!("define Cohort:\n  {}", expression(e, mapping)?)];
    let mut groups = Vec::new();
    for (key, count) in &mapping.counts {
        let name = format!("count_{key}");
        defines.push(members_define(
            &name,
            &count.resource_type,
            &count.filter,
            e,
            mapping,
        )?);
        groups.push(group(key, &count.resource_type, name, None));
    }
    for (key, s) in &mapping.stratifiers {
        let members = format!("stratifier_{key}_members");
        let value = format!("stratifier_{key}");
        defines.push(members_define(
            &members,
            &s.resource_type,
            &s.filter,
            e,
            mapping,
        )?);
        defines.push(if s.resource_type == "Patient" {
            format!(
                "define \"{value}\":\n  {}",
                s.cql.replace("{{this}}", "Patient")
            )
        } else {
            format!(
                "define function \"{value}\"(M FHIR.{}):\n  {}",
                s.resource_type,
                s.cql.replace("{{this}}", "M")
            )
        });
        groups.push(group(key, &s.resource_type, members, Some(&value)));
    }
    let cql = format!(
        "library SiteProtocol version '1.0.0'\nusing FHIR version '4.0.0'\ninclude FHIRHelpers version '4.0.0'\n\ncontext Patient\n\n{}\n",
        defines.join("\n\n")
    );

    let library_url = format!("urn:uuid:{}", Uuid::new_v4());
    let measure_url = format!("urn:uuid:{}", Uuid::new_v4());
    let library = json!({
        "resourceType": "Library",
        "status": "active",
        "url": library_url,
        "type": {"coding": [{"system": "http://terminology.hl7.org/CodeSystem/library-type", "code": "logic-library"}]},
        "content": [{"contentType": "text/cql", "data": BASE64.encode(cql)}],
    });

    let measure = json!({
        "resourceType": "Measure",
        "status": "active",
        "url": measure_url,
        "library": [library_url],
        "subjectCodeableConcept": {"coding": [{"system": "http://hl7.org/fhir/resource-types", "code": "Patient"}]},
        "scoring": {"coding": [{"system": "http://terminology.hl7.org/CodeSystem/measure-scoring", "code": "cohort"}]},
        "group": groups,
    });

    Ok((library, measure))
}

#[derive(Serialize, Default)]
pub struct Response {
    pub counts: BTreeMap<String, u64>,
    pub stratifiers: BTreeMap<String, BTreeMap<String, u64>>,
}

/// Read the counts out of a MeasureReport. Groups with a stratifier only
/// report its strata; members without a value (the "null" stratum) are left
/// out.
pub fn read_report(report: &str) -> Result<Response, Error> {
    let report: Value = serde_json::from_str(report)
        .map_err(|e| Error::Failed(format!("unreadable MeasureReport: {e}")))?;
    let mut response = Response::default();
    let count = |v: &Value| v["population"][0]["count"].as_u64().unwrap_or(0);

    for group in report["group"].as_array().into_iter().flatten() {
        let Some(key) = group["code"]["text"].as_str() else {
            continue;
        };
        let Some(stratifier) = group["stratifier"].get(0) else {
            response.counts.insert(key.into(), count(group));
            continue;
        };
        let mut counts = BTreeMap::new();
        for stratum in stratifier["stratum"].as_array().into_iter().flatten() {
            match stratum["value"]["text"].as_str() {
                None | Some("null") => {}
                Some(value) => *counts.entry(value.into()).or_default() += count(stratum),
            }
        }
        response.stratifiers.insert(key.into(), counts);
    }
    Ok(response)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::site_protocol::{ast::Query, mapping};

    fn cql(query: &str) -> Result<String, Error> {
        let query: Query = serde_json::from_str(query).unwrap();
        let (library, _) = generate(&query.expression, &mapping::load("dktk").unwrap())?;
        let data = library["content"][0]["data"].as_str().unwrap();
        Ok(String::from_utf8(BASE64.decode(data).unwrap()).unwrap())
    }

    fn rejection(query: &str) -> String {
        match cql(query) {
            Err(Error::Rejected(message)) => message,
            _ => panic!("not rejected"),
        }
    }

    #[test]
    fn generates_temporal_chain() {
        let cql = cql(r#"{"expression": {"kind": "and", "operands": [
            {"kind": "subject", "operand": {"kind": "values", "field": "gender", "values": ["female"]}},
            {"kind": "temporal",
             "first": {"kind": "exists", "scope": "diagnosis",
                       "operand": {"kind": "range", "field": "age_at_diagnosis", "min": 50}},
             "steps": [{"operator": "within_days", "days": 30,
                        "operand": {"kind": "exists", "scope": "specimen"}}]}
        ]}}"#)
        .unwrap();
        assert!(cql.contains("Patient.gender in {'female'}"));
        assert!(cql.contains("between 50 and maximum Decimal"));
        assert!(cql.contains("days between"));
        assert!(cql.contains("<= 30"));
        // The specimen chip has no criteria, so every specimen counts.
        assert!(cql.contains("[Specimen] M where ((true))"));
    }

    #[test]
    fn rejects_bad_queries() {
        assert!(
            rejection(r#"{"expression": {"kind": "exists", "scope": "nope"}}"#)
                .contains("unknown scope")
        );
        assert!(rejection(
            r#"{"expression": {"kind": "subject", "operand": {"kind": "range", "field": "gender"}}}"#
        )
        .contains("not of type Range"));
        assert!(rejection(
            r#"{"expression": {"kind": "temporal", "first": {"kind": "exists", "scope": "tnm"},
                "steps": [{"operator": "then", "operand": {"kind": "exists", "scope": "specimen"}}]}}"#
        )
        .contains("temporal"));
        assert!(serde_json::from_str::<Query>(
            r#"{"expression": {"kind": "subject", "operand":
                {"kind": "values", "field": "gender", "values": [], "negate": true}}}"#
        )
        .is_err());
    }

    #[test]
    fn reads_report() {
        let report = r#"{"group": [
            {"code": {"text": "patients"}, "population": [{"count": 3}]},
            {"code": {"text": "age"}, "population": [{"count": 3}], "stratifier": [
                {"code": [{"text": "age"}], "stratum": [
                    {"value": {"text": "20-39"}, "population": [{"count": 2}]},
                    {"value": {"text": "null"}, "population": [{"count": 1}]}]}]}]}"#;
        assert_eq!(
            serde_json::to_value(read_report(report).unwrap()).unwrap(),
            json!({"counts": {"patients": 3}, "stratifiers": {"age": {"20-39": 2}}})
        );
    }
}
