use tracing::{error, trace};

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::ast;
use crate::errors::FocusError;

pub static CRITERION_CATEGORY: Lazy<HashMap<&str, &u8>> = Lazy::new(|| {
    //0 - patient criterion, 1 - image_criterion
    let mut map: HashMap<&'static str, &'static u8> = HashMap::new();
    map.insert("SNOMEDCT263495000", &0);
    map.insert("SNOMEDCT439401001", &0);
    map.insert("RID10311", &1);
    map.insert("SNOMEDCT123037004", &1);
    map.insert("C25392", &1);

    map
});

pub static CRITERION_SNIPPET: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert(
        "SNOMEDCT263495000",
        " (cancerpatient.birthsexeucaim = '?') ",
    );
    map.insert("SNOMEDCT439401001", " (EXISTS (SELECT * FROM primarycancercondition WHERE primarycancercondition.patientidentifier = cancerpatient.identifier AND primarycancerconditioneucaim = '?')) ");
    map.insert("RID10311", " TRUE ");
    map.insert("SNOMEDCT123037004", " EXISTS (SELECT FROM imageseries WHERE bodypartexamined = '?' AND imageseries.imagestudyidentifier = imagestudy.id) ");
    map.insert("C25392", " TRUE ");

    map
});

pub static CRITERION: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    map.insert("SNOMEDCT248153007", "COM1000180");
    map.insert("SNOMEDCT248152002", "COM1000177");
    map.insert("SNOMEDCT74964007", ""); //can't find it in concepts
    map.insert("SNOMEDCT261665006", "COM1001289");
    map.insert("SNOMEDCT363406005", "CLIN1000057"); // colon cancer
    map.insert("SNOMEDCT254837009", "CLIN1000060"); // breast cancer
    map.insert("SNOMEDCT363358000", "CLIN1000065"); // lung cancer
    map.insert("SNOMEDCT363484005", "CLIN1000087"); // pelvis cancer
    map.insert("SNOMEDCT399068003", "CLIN1000075"); // prostate cancer
    map.insert("RID10312", "MR"); // Modalities are integers in the DB! Search impossible until clarified!
    map.insert("RID10337", "PET"); // Modalities are integers in the DB! Search impossible until clarified!
    map.insert("RID10334", "SPECT"); // Modalities are integers in the DB! Search impossible until clarified!
    map.insert("RID10321", "CT"); // Modalities are integers in the DB! Search impossible until clarified!
    map.insert("SNOMEDCT76752008", "BP1000136");
    map.insert("SNOMEDCT71854001", "BP1000257");
    map.insert("SNOMEDCT39607008", "BP1000113");
    map.insert("SNOMEDCT12921003", "BP1000092");
    map.insert("SNOMEDCT41216001", "BP1000021");
    map.insert("C200140", "Siemens"); // can't find it in concepts
    map.insert("birnlex_3066", "Siemens"); // can't find it in concepts
    map.insert("birnlex_12833", "General%20Electric"); // can't find it in concepts
    map.insert("birnlex_3065", "Philips"); // can't find it in concepts
    map.insert("birnlex_3067", "Toshiba"); // can't find it in concepts

    map
});

pub fn build_eucaim_sql_query(ast: ast::Ast) -> Result<String, FocusError> {
    const BEFORE_IMAGE_CRITERIA: &str = r#"SELECT dataset.identifier id, dataset.title name, dataset.description, COUNT(cancerpatient.*)::int4 subjects_count, COALESCE(SUM ((SELECT COUNT(imagestudy.*) FROM imagingprocedure JOIN imagestudy ON imagingprocedure.procedureidentifier = imagestudy.imagingprocedureidentifier::varchar(255) WHERE TRUE "#;
    const BETWEEN_CRITERIA: &str = r#" GROUP BY imagingprocedure.patientidentifier HAVING imagingprocedure.patientidentifier = cancerpatient.identifier)), 0)::int4 studies_count FROM dataset JOIN cancerpatient ON cancerpatient.datasetidentifier = dataset.identifier WHERE TRUE "#;
    const AFTER_PATIENT_CRITERIA: &str =
        r#" GROUP BY cancerpatient.datasetidentifier, dataset.identifier;"#;

    let mut criteria: HashMap<u8, String> = HashMap::new(); // 0 - patient criteria; 1 - image criteria
    criteria.insert(0, "".to_string());
    criteria.insert(1, "".to_string());

    let children = ast.ast.children;

    if children.len() > 1 {
        error!("Too many children! AND/OR queries not supported."); // I'm just gonna do it like this for now because if more subqueries we lose providers that use the API as well
        return Err(FocusError::EucaimQueryGenerationError);
    }

    for child in children {
        // will be either 0 or 1
        match child {
            ast::Child::Operation(operation) => {
                if operation.operand == ast::Operand::Or {
                    error!("OR found as first level operator");
                    return Err(FocusError::EucaimQueryGenerationError);
                }
                for grandchild in operation.children {
                    match grandchild {
                        ast::Child::Operation(operation) => {
                            if operation.operand == ast::Operand::And {
                                error!("AND found as second level operator");
                                return Err(FocusError::EucaimQueryGenerationError);
                            }
                            let greatgrandchildren = operation.children;
                            if greatgrandchildren.len() > 1 {
                                error!("Too many children! OR operator between criteria of the same type not supported.");
                                return Err(FocusError::EucaimQueryGenerationError);
                            }

                            for greatgrandchild in greatgrandchildren {
                                match greatgrandchild {
                                    ast::Child::Operation(_) => {
                                        error!(
                                            "Search tree has too many levels. Query not supported"
                                        );
                                        return Err(FocusError::EucaimQueryGenerationError);
                                    }
                                    ast::Child::Condition(condition) => {
                                        let category_maybe =
                                            CRITERION_CATEGORY.get(&(condition.key).as_str());
                                        if let Some(category) = category_maybe {
                                            let category_index = **category;
                                            let snippet_maybe =
                                                CRITERION_SNIPPET.get(&(condition.key).as_str());
                                            if let Some(snippet) = snippet_maybe {
                                                match condition.value {
                                                    ast::ConditionValue::String(value) => {
                                                        let criterion_maybe =
                                                            CRITERION.get(&(value).as_str());
                                                        if let Some(criterion) = criterion_maybe {
                                                            let crit_maybe =
                                                                criteria.get(&category_index);
                                                            if let Some(crit) = crit_maybe {
                                                                let mut new_crit: String =
                                                                    (*(crit.clone())).to_string();
                                                                if crit.len() > 0 {
                                                                    new_crit = new_crit + " AND ";
                                                                }
                                                                new_crit = new_crit
                                                                    + snippet
                                                                        .replace("?", criterion)
                                                                        .as_str();
                                                                criteria.insert(
                                                                    category_index,
                                                                    new_crit,
                                                                );
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        error!("The only supported condition value type is string");
                                                        return Err(
                                                            FocusError::EucaimQueryGenerationError,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        ast::Child::Condition(_) => {
                            // must be operation
                            error!("Condition found as second level child");
                            return Err(FocusError::EucaimQueryGenerationError);
                        }
                    }
                }
            }
            ast::Child::Condition(_) => {
                // must be operation
                error!("Condition found as first level child");
                return Err(FocusError::EucaimQueryGenerationError);
            }
        }
    }

    let mut patient_criteria = criteria.get(&0).cloned().unwrap_or_default();
    if patient_criteria.len() > 0 {
        patient_criteria = " AND (".to_string() + patient_criteria.as_str() + ")";
    }
    let mut image_criteria = criteria.get(&1).cloned().unwrap_or_default();
    if image_criteria.len() > 0 {
        image_criteria = " AND (".to_string() + image_criteria.as_str() + ")";
    }

    let sql = BEFORE_IMAGE_CRITERIA.to_string()
        + image_criteria.as_str()
        + BETWEEN_CRITERIA
        + patient_criteria.as_str()
        + AFTER_PATIENT_CRITERIA;

    trace!("{}", &sql);
    Ok(sql)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions;

    const EMPTY: &str =
        r#"{"ast":{"children":[],"operand":"OR"}, "id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}"#;

    const SQL_NO_CRITERIA_SELECTED: &str = r#"SELECT dataset.identifier id, dataset.title name, dataset.description, COUNT(cancerpatient.*)::int4 subjects_count, COALESCE(SUM ((SELECT COUNT(imagestudy.*) FROM imagingprocedure JOIN imagestudy ON imagingprocedure.procedureidentifier = imagestudy.imagingprocedureidentifier::varchar(255) WHERE TRUE  GROUP BY imagingprocedure.patientidentifier HAVING imagingprocedure.patientidentifier = cancerpatient.identifier)), 0)::int4 studies_count FROM dataset JOIN cancerpatient ON cancerpatient.datasetidentifier = dataset.identifier WHERE TRUE  GROUP BY cancerpatient.datasetidentifier, dataset.identifier;"#;

    const JUST_RIGHT: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248153007"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT399068003"}],"operand":"OR"},{"children":[{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10312"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT76752008"}],"operand":"OR"},{"children":[{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3065"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"66b8bbf4-ded2-4f94-87ab-3a3ca2f4edc0__search__66b8bbf4-ded2-4f94-87ab-3a3ca2f4edc0"}"#;

    const JUST_RIGHT_SQL_TEMP: &str = r#"SELECT dataset.identifier id, dataset.title name, dataset.description, COUNT(cancerpatient.*)::int4 subjects_count, COALESCE(SUM ((SELECT COUNT(imagestudy.*) FROM imagingprocedure JOIN imagestudy ON imagingprocedure.procedureidentifier = imagestudy.imagingprocedureidentifier::varchar(255) WHERE TRUE  AND ( TRUE  AND  EXISTS (SELECT FROM imageseries WHERE bodypartexamined = 'BP1000136' AND imageseries.imagestudyidentifier = imagestudy.id)  AND  TRUE ) GROUP BY imagingprocedure.patientidentifier HAVING imagingprocedure.patientidentifier = cancerpatient.identifier)), 0)::int4 studies_count FROM dataset JOIN cancerpatient ON cancerpatient.datasetidentifier = dataset.identifier WHERE TRUE  AND ( (cancerpatient.birthsexeucaim = 'COM1000180')  AND  (EXISTS (SELECT * FROM primarycancercondition WHERE primarycancercondition.patientidentifier = cancerpatient.identifier AND primarycancerconditioneucaim = 'CLIN1000075')) ) GROUP BY cancerpatient.datasetidentifier, dataset.identifier;"#;

    const TOO_MUCH: &str = r#"{"ast":{"children":[{"children":[{"children":[{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248153007"},{"key":"SNOMEDCT263495000","system":"","type":"EQUALS","value":"SNOMEDCT248152002"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT399068003"},{"key":"SNOMEDCT439401001","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT254837009"}],"operand":"OR"},{"children":[{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10312"},{"key":"RID10311","system":"urn:oid:2.16.840.1.113883.6.256","type":"EQUALS","value":"RID10337"}],"operand":"OR"},{"children":[{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT76752008"},{"key":"SNOMEDCT123037004","system":"urn:snomed-org/sct","type":"EQUALS","value":"SNOMEDCT41216001"}],"operand":"OR"},{"children":[{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3065"},{"key":"C25392","system":"http://bioontology.org/projects/ontologies/birnlex","type":"EQUALS","value":"birnlex_3067"}],"operand":"OR"}],"operand":"AND"}],"operand":"OR"},"id":"c57e075c-19de-4c5a-ba9c-b8f697a98dfc__search__c57e075c-19de-4c5a-ba9c-b8f697a98dfc"}"#;

    #[test]
    fn test_build_sql_empty() {
        let sql = build_eucaim_sql_query(serde_json::from_str(EMPTY).unwrap()).unwrap();
        pretty_assertions::assert_eq!(sql, SQL_NO_CRITERIA_SELECTED);
    }

    #[test]
    fn test_build_sql_just_right() {
        let sql = build_eucaim_sql_query(serde_json::from_str(JUST_RIGHT).unwrap()).unwrap();
        pretty_assertions::assert_eq!(sql, JUST_RIGHT_SQL_TEMP);
    }

    #[test]
    fn test_build_sql_too_much() {
        assert!(build_eucaim_sql_query(serde_json::from_str(TOO_MUCH).unwrap(),).is_err());
    }
}
