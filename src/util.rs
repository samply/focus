use serde_json::Value;
use std::collections::HashMap;


pub(crate) fn get_json_field(json_string: &str, field: &str) -> Result<Value, serde_json::Error> {
    let json: Value = serde_json::from_str(json_string)?;
    Ok(json[field].clone())
}

pub(crate) fn replace_cql(decoded_library: impl Into<String>) -> String {
    let replace_map: HashMap<&str, &str> =
        [
            ("BORSCHTSCH_GENDER_STRATIFIER", 
            "define Gender:
             if (Patient.gender is null) then 'unknown' else Patient.gender"
            ),
            ("BORSCHTSCH_SAMPLE_TYPE_STRATIFIER", "define function SampleType(specimen FHIR.Specimen):\n    case FHIRHelpers.ToCode(specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').first())\n        when Code 'plasma-edta' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-citrat' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-heparin' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-cell-free' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma-other' from SampleMaterialType then 'blood-plasma' \n        when Code 'plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-formalin' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'normal-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'other-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tumor-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'normal-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'other-tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-paxgene-or-else' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid' from SampleMaterialType then 'liquid-other' \n        when Code 'tissue' from SampleMaterialType then 'tissue-other' \n        when Code 'serum' from SampleMaterialType then 'blood-serum' \n        when Code 'cf-dna' from SampleMaterialType then 'dna' \n        when Code 'g-dna' from SampleMaterialType then 'dna' \n        when Code 'blood-plasma' from SampleMaterialType then 'blood-plasma' \n        when Code 'tissue-ffpe' from SampleMaterialType then 'tissue-ffpe' \n        when Code 'tissue-frozen' from SampleMaterialType then 'tissue-frozen' \n        when Code 'tissue-other' from SampleMaterialType then 'tissue-other' \n        when Code 'derivative-other' from SampleMaterialType then 'derivative-other' \n        when Code 'liquid-other' from SampleMaterialType then 'liquid-other' \n        when Code 'blood-serum' from SampleMaterialType then 'blood-serum' \n        when Code 'dna' from SampleMaterialType then 'dna' \n        when Code 'buffy-coat' from SampleMaterialType then 'buffy-coat' \n        when Code 'urine' from SampleMaterialType then 'urine' \n        when Code 'ascites' from SampleMaterialType then 'ascites' \n        when Code 'saliva' from SampleMaterialType then 'saliva' \n        when Code 'csf-liquor' from SampleMaterialType then 'csf-liquor' \n        when Code 'bone-marrow' from SampleMaterialType then 'bone-marrow' \n        when Code 'peripheral-blood-cells-vital' from SampleMaterialType then 'peripheral-blood-cells-vital' \n        when Code 'stool-faeces' from SampleMaterialType then 'stool-faeces' \n        when Code 'rna' from SampleMaterialType then 'rna' \n        when Code 'whole-blood' from SampleMaterialType then 'whole-blood' \n        when Code 'swab' from SampleMaterialType then 'swab' \n        when Code 'dried-whole-blood' from SampleMaterialType then 'dried-whole-blood' \n        when null  then 'Unknown'\n        else 'Unknown'\n    end"),
            ("BORSCHTSCH_CUSTODIAN_STRATIFIER", "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)"),
            ("BORSCHTSCH_DIAGNOSIS_STRATIFIER", "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n"),
            ("BORSCHTSCH_AGE_STRATIFIER", "define AgeClass:\n     (AgeInYears() div 10) * 10"),
            ("BORSCHTSCH_DEF_SPECIMEN", "define Specimen:"),
            ("BORSCHTSCH_DEF_IN_INITIAL_POPULATION", "define InInitialPopulation:")
        ].into();

    let mut decoded_library = decoded_library.into();

    for (key, value) in replace_map {
        let replacement_value = value.to_string() + "\n";
        decoded_library = decoded_library.replace(key, &replacement_value[..]);
    }

    decoded_library
}

pub(crate) fn is_cql_tampered_with(decoded_library: impl Into<String>) -> bool {
    let decoded_library = decoded_library.into();
    decoded_library.contains("define")
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_json_field_success() {
        let json_string = r#"
            {
                "name": "FHIRy McFHIRFace",
                "age": 47,
                "address": {
                    "street": "Brückenkopfstrasse 1",
                    "city": "Heidelberg",
                    "state": "BW",
                    "zip": "69120"
                }
            }
        "#;
        let expected_result = serde_json::json!({
            "street": "Brückenkopfstrasse 1",
            "city": "Heidelberg",
            "state": "BW",
            "zip": "69120"
        });

        // Call the function and assert that it returns the expected result
        let result = get_json_field(json_string, "address");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_result);
    }

    #[test]
    fn test_get_json_field_nonexistent_field() {
        let json_string = r#"
            {
                "name": "FHIRy McFHIRFace",
                "age": 47,
                "address": {
                    "street": "Brückenkopfstrasse 1",
                    "city": "Heidelberg",
                    "state": "BW",
                    "zip": "69120"
                }
            }
        "#;

        // Call the function and assert that it returns json null
        let result = get_json_field(json_string, "phone");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(null));
    }

    #[test]
    fn test_get_json_field_invalid_json() {
        let json_string = r#"{"name": "FHIRy McFHIRFace", "age": 47"#;

        // Call the function and assert that it returns an error
        let result = get_json_field(json_string, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_cql_tampered_with_true() {
        let decoded_library =
            "define Gender:\n if (Patient.gender is null) then 'unknown' else Patient.gender \n";
        assert!(is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_is_cql_tampered_with_false() {
        let decoded_library = "context Patient\nBORSCHTSCH_GENDER_STRATIFIER";
        assert!(!is_cql_tampered_with(decoded_library));
    }

    #[test]
    fn test_replace_cql() {
        let decoded_library = "BORSCHTSCH_GENDER_STRATIFIER";
        let expected_result = "define Gender:\n             if (Patient.gender is null) then 'unknown' else Patient.gender\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_CUSTODIAN_STRATIFIER";
        let expected_result = "define Custodian:\n First(from Specimen.extension E\n where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'\n return (E.value as Reference).identifier.value)\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DIAGNOSIS_STRATIFIER";
        let expected_result = "define Diagnosis:\n if InInitialPopulation then [Condition] else {} as List<Condition> \n define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):\n Coalesce(condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(), specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first(), condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first())\n\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_AGE_STRATIFIER";
        let expected_result = "define AgeClass:\n     (AgeInYears() div 10) * 10\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DEF_SPECIMEN";
        let expected_result = "define Specimen:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "BORSCHTSCH_DEF_IN_INITIAL_POPULATION";
        let expected_result = "define InInitialPopulation:\n";
        assert_eq!(replace_cql(decoded_library), expected_result);

        let decoded_library = "INVALID_KEY";
        let expected_result = "INVALID_KEY";
        assert_eq!(replace_cql(decoded_library), expected_result);
    }


}
