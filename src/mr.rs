use crate::{
    errors::FocusError,
    transformed::{Facets, Transformed},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasureReport {
    pub date: String,
    pub extension: Vec<Extension>,
    pub group: Vec<Group>,
    pub id: Option<String>,
    pub measure: String,
    pub meta: Option<Value>,
    pub period: Period,
    pub resource_type: String,
    pub status: String,
    pub type_: String, //because "type" is a reserved keyword
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    pub code: Code,
    pub population: Vec<Population>,
    pub stratifier: Vec<Stratifier>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Population {
    pub code: PopulationCode,
    pub count: u64,
    pub subject_results: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PopulationCode {
    pub coding: Vec<Coding>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Coding {
    pub code: String,
    pub system: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Period {
    pub end: String,
    pub start: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValueQuantity {
    pub code: String,
    pub system: String,
    pub unit: String,
    pub value: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ValueRatio {
    pub denominator: Value,
    pub numerator: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    pub url: String,
    pub value_quantity: Option<ValueQuantity>,
    pub value_ratio: Option<ValueRatio>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Code {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StratumValue {
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stratum {
    pub population: Vec<Population>,
    pub value: StratumValue,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stratifier {
    pub code: Vec<Code>,
    pub stratum: Option<Vec<Stratum>>,
}

pub fn transform_lens(measure_report: MeasureReport) -> Result<Transformed, FocusError> {
    //let mut stratifier_groups: StratifierGroups = StratifierGroups::new();
    let mut transformed: Transformed = Transformed {
        stratifiers: Default::default(),
        totals: Default::default(),
    };

    //let mut stratifiers = Stratifiers::new();

    for g in &measure_report.group {
        transformed
            .totals
            .insert(g.code.text.clone(), g.population[0].count);
        for s in &g.stratifier {
            let mut facets = Facets::new();

            let facets_key = s
                .code
                .first()
                .ok_or_else(|| FocusError::ParsingError("Missing facet key".into()))?
                .text
                .clone();
            if let Some(strata) = &s.stratum {
                for stratum in strata {
                    let stratum_key = stratum.value.text.clone();
                    let value = stratum
                        .population
                        .first()
                        .ok_or_else(|| FocusError::ParsingError("Missing facet count".into()))?
                        .count;

                    facets.insert(stratum_key, value);
                }
            }
            transformed.stratifiers.insert(facets_key, facets);
        }
    }
    Ok(transformed)
}

#[cfg(test)]
mod test {

    use super::*;

    const EXAMPLE_MEASURE_REPORT_BBMRI: &str =
        include_str!("../resources/test/measure_report_bbmri.json");
    const STRATIFIER_GROUPS_BBMRI: &str =
        include_str!("../resources/test/stratifier_groups_bbmri.json");
    const EXAMPLE_MEASURE_REPORT_DKTK: &str =
        include_str!("../resources/test/measure_report_dktk.json");
    const STRATIFIER_GROUPS_DKTK: &str =
        include_str!("../resources/test/stratifier_groups_dktk.json");

    #[test]
    fn test_extract_facets_bbmri() {
        let measure_report: MeasureReport =
            serde_json::from_str(&EXAMPLE_MEASURE_REPORT_BBMRI).expect("Can't be deserialized");

        let stratifiers =
            transform_lens(measure_report).expect("what, no proper stratifier groups");

        let stratifiers_json = serde_json::to_string(&stratifiers).expect("Should be JSON");

        pretty_assertions::assert_eq!(STRATIFIER_GROUPS_BBMRI, stratifiers_json);
    }

    #[test]
    fn test_extract_facets_dktk() {
        let measure_report: MeasureReport =
            serde_json::from_str(&EXAMPLE_MEASURE_REPORT_DKTK).expect("Can't be deserialized");

        let stratifiers =
            transform_lens(measure_report).expect("what, no proper stratifier groups");

        let stratifiers_json = serde_json::to_string(&stratifiers).expect("Should be JSON");

        pretty_assertions::assert_eq!(STRATIFIER_GROUPS_DKTK, stratifiers_json);
    }
}
