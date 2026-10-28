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

fn scoped_predicate(exists: &Exists, scope: &Scope, alias: &str) -> Result<Option<String>, Error> {
    let place = format!("scope \"{}\"", exists.scope);
    exists
        .operand
        .as_ref()
        .map(|p| predicate(p, &scope.fields, &place, alias))
        .transpose()
}

fn exists(exists: &Exists, mapping: &Mapping) -> Result<String, Error> {
    let scope = scope(mapping, &exists.scope)?;
    Ok(match scoped_predicate(exists, scope, "Q")? {
        None => format!("exists({})", scope.retrieve()),
        Some(p) => format!("exists(from {} Q where {p})", scope.retrieve()),
    })
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
        let mut predicates: Vec<String> = scoped_predicate(exists, scope, &alias)?
            .into_iter()
            .collect();
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
            "exists(from {} {alias} where {})",
            scope.retrieve(),
            predicates.join("\n  and ")
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

/// The query's criteria on Specimen scopes, so the specimen count only
/// includes specimens a chip asked for. Negated parts are left out.
fn specimen_criteria<'a>(e: &'a Expression, out: &mut Vec<&'a Exists>) {
    match e {
        Expression::And { operands } | Expression::Or { operands } => {
            operands.iter().for_each(|o| specimen_criteria(o, out))
        }
        Expression::Exists(x) => out.push(x),
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

fn population_define(
    key: &str,
    resource_type: &str,
    e: &Expression,
    mapping: &Mapping,
) -> Result<String, Error> {
    let cohort = mapping.patient_population();
    if resource_type == "Patient" {
        return Ok(format!(
            "define \"population_{key}\":\n  {}",
            expression(e, mapping)?
        ));
    }
    let mut source = format!("[{resource_type}]");
    if resource_type == "Specimen" {
        let mut criteria = Vec::new();
        specimen_criteria(e, &mut criteria);
        criteria.retain(|x| {
            mapping
                .scopes
                .get(&x.scope)
                .is_some_and(|s| s.resource_type == "Specimen")
        });
        if !criteria.is_empty() {
            let parts = criteria
                .iter()
                .map(|x| {
                    let scope = scope(mapping, &x.scope)?;
                    Ok(format!(
                        "({})",
                        scoped_predicate(x, scope, "S")?.unwrap_or("true".into())
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?;
            source = format!("[Specimen] S where {}", parts.join(" or "));
        }
    }
    Ok(format!(
        "define \"population_{key}\":\n  if \"population_{cohort}\" then {source} else {{}} as List<FHIR.{resource_type}>"
    ))
}

fn stratifier_define(key: &str, mapping: &Mapping) -> String {
    let stratifier = &mapping.stratifiers[key];
    let resource_type = &mapping.populations[&stratifier.population].resource_type;
    let this = if resource_type == "Patient" {
        "Patient"
    } else {
        "Member"
    };
    let body = match (&stratifier.cql, &stratifier.buckets) {
        (Some(cql), _) => cql.replace("{{this}}", this),
        (None, buckets) => {
            let whens: String = buckets
                .iter()
                .flatten()
                .map(|b| {
                    format!(
                        "\n    when {} then '{}'",
                        b.cql.replace("{{this}}", this),
                        escape(&b.key)
                    )
                })
                .collect();
            format!("case{whens}\n    else null\n  end")
        }
    };
    if resource_type == "Patient" {
        format!("define \"stratifier_{key}\":\n  {body}")
    } else {
        format!("define function \"stratifier_{key}\"(Member FHIR.{resource_type}):\n  {body}")
    }
}

/// Build the Library and Measure that evaluate `e` against the mapping.
pub fn generate(e: &Expression, mapping: &Mapping) -> Result<(Value, Value), Error> {
    let mut defines = Vec::new();
    for (key, population) in &mapping.populations {
        defines.push(population_define(
            key,
            &population.resource_type,
            e,
            mapping,
        )?);
    }
    for key in mapping.stratifiers.keys() {
        defines.push(stratifier_define(key, mapping));
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

    let groups: Vec<Value> = mapping
        .populations
        .iter()
        .map(|(key, population)| {
            let stratifiers: Vec<Value> = mapping
                .stratifiers
                .iter()
                .filter(|(_, s)| &s.population == key)
                .map(|(s, _)| json!({
                    "code": {"text": s},
                    "criteria": {"language": "text/cql-identifier", "expression": format!("stratifier_{s}")},
                }))
                .collect();
            let mut group = json!({
                "code": {"text": key},
                "population": [{
                    "code": {"coding": [{"system": "http://terminology.hl7.org/CodeSystem/measure-population", "code": "initial-population"}]},
                    "criteria": {"language": "text/cql-identifier", "expression": format!("population_{key}")},
                }],
                "stratifier": stratifiers,
            });
            if population.resource_type != "Patient" {
                group["extension"] = json!([{
                    "url": "http://hl7.org/fhir/us/cqfmeasures/StructureDefinition/cqfm-populationBasis",
                    "valueCode": population.resource_type,
                }]);
            }
            group
        })
        .collect();

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
    populations: BTreeMap<String, u64>,
    stratifiers: BTreeMap<String, BTreeMap<String, u64>>,
}

/// Read the counts out of a MeasureReport. Bucket stratifiers report every
/// bucket, zero or not.
pub fn read_report(report: &str, mapping: &Mapping) -> Result<Response, Error> {
    let report: Value = serde_json::from_str(report)
        .map_err(|e| Error::Failed(format!("unreadable MeasureReport: {e}")))?;
    let mut response = Response::default();
    let count = |v: &Value| v["population"][0]["count"].as_u64().unwrap_or(0);

    for group in report["group"].as_array().into_iter().flatten() {
        let Some(population) = group["code"]["text"].as_str() else {
            continue;
        };
        response.populations.insert(population.into(), count(group));

        for stratifier in group["stratifier"].as_array().into_iter().flatten() {
            let Some(key) = stratifier["code"][0]["text"].as_str() else {
                continue;
            };
            let buckets = mapping
                .stratifiers
                .get(key)
                .and_then(|s| s.buckets.as_ref());
            let mut counts: BTreeMap<String, u64> = buckets
                .into_iter()
                .flatten()
                .map(|b| (b.key.clone(), 0))
                .collect();
            for stratum in stratifier["stratum"].as_array().into_iter().flatten() {
                let Some(value) = stratum["value"]["text"].as_str() else {
                    continue;
                };
                if buckets.is_some() && !counts.contains_key(value) {
                    continue;
                }
                *counts.entry(value.into()).or_default() += count(stratum);
            }
            response.stratifiers.insert(key.into(), counts);
        }
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
        assert!(cql.contains("[Specimen] S where (true)"));
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
        let report = r#"{"group": [{"code": {"text": "patients"}, "population": [{"count": 3}],
            "stratifier": [
                {"code": [{"text": "age"}], "stratum": [
                    {"value": {"text": "20-39"}, "population": [{"count": 2}]},
                    {"value": {"text": "null"}, "population": [{"count": 1}]}]},
                {"code": [{"text": "gender"}], "stratum": [
                    {"value": {"text": "female"}, "population": [{"count": 3}]}]}]}]}"#;
        let response = read_report(report, &mapping::load("dktk").unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "populations": {"patients": 3},
                "stratifiers": {
                    "age": {"0-19": 0, "20-39": 2, "40-59": 0, "60-79": 0, "80+": 0},
                    "gender": {"female": 3},
                },
            })
        );
    }
}
