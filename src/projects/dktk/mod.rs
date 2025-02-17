use std::collections::HashMap;

use indexmap::IndexSet;

use super::{CriterionRole, Project, ProjectName};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Dktk;

// TODO: Include entries from shared
impl Project for Dktk {
    fn append_code_lists(&self, map: &mut HashMap<&'static str, &'static str>) {
        map.extend([
            ("icd10", "http://fhir.de/CodeSystem/bfarm/icd-10-gm"),
            ("loinc", "http://loinc.org"),
            (
                "gradingcs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/GradingCS",
            ),
            ("ops", "http://fhir.de/CodeSystem/bfarm/ops"),
            ("morph", "urn:oid:2.16.840.1.113883.6.43.1"),
            ("lokalisation_icd_o_3", "urn:oid:2.16.840.1.113883.6.43.1"),
            (
                "bodySite",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SeitenlokalisationCS",
            ),
            (
                "Therapieart",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SYSTTherapieartCS",
            ),
            (
                "specimentype",
                "https://fhir.bbmri.de/CodeSystem/SampleMaterialType",
            ),
            (
                "uiccstadiumcs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/UiccstadiumCS",
            ),
            (
                "lokalebeurteilungresidualstatuscs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/LokaleBeurteilungResidualstatusCS",
            ),
            (
                "gesamtbeurteilungtumorstatuscs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/GesamtbeurteilungTumorstatusCS",
            ),
            (
                "verlauflokalertumorstatuscs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VerlaufLokalerTumorstatusCS",
            ),
            (
                "verlauftumorstatuslymphknotencs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VerlaufTumorstatusLymphknotenCS",
            ),
            (
                "verlauftumorstatusfernmetastasencs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VerlaufTumorstatusFernmetastasenCS",
            ),
            (
                "vitalstatuscs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VitalstatusCS",
            ),
            (
                "jnucs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/JNUCS",
            ),
            (
                "fmlokalisationcs",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/FMLokalisationCS",
            ),
            (
                "TNMTCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMTCS",
            ),
            (
                "TNMNCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMNCS",
            ),
            (
                "TNMMCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMMCS",
            ),
            (
                "TNMySymbolCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMySymbolCS",
            ),
            (
                "TNMrSymbolCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMrSymbolCS",
            ),
            (
                "TNMmSymbolCS",
                "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/TNMmSymbolCS",
            ),
            ("molecularMarker", "http://www.genenames.org"),
            ("BBMRI_icd10", "http://hl7.org/fhir/sid/icd-10"),
            ("BBMRI_icd10gm", "http://fhir.de/CodeSystem/dimdi/icd-10-gm"),
            (
                "BBMRI_SampleMaterialType",
                "https://fhir.bbmri.de/CodeSystem/SampleMaterialType",
            ), //specimentype
            (
                "BBMRI_StorageTemperature",
                "https://fhir.bbmri.de/CodeSystem/StorageTemperature",
            ),
            (
                "BBMRI_SmokingStatus",
                "http://hl7.org/fhir/uv/ips/ValueSet/current-smoking-status-uv-ips",
            ),
        ]);
    }

    fn append_observation_loinc_codes(&self, _map: &mut HashMap<&'static str, &'static str>) {}

    fn append_criterion_code_lists(&self, map: &mut HashMap<&str, Vec<&str>>) {
        for key in ["OP", "ST", "CH", "HO", "IM", "KM"] {
            map.insert(key, vec!["Therapieart"]);
        }
        for key in ["histology", "48005-3", "81290-9", "81248-7", "81249-5"] {
            map.insert(key, vec!["loinc"]);
        }

        map.extend([
            ("diagnosis", vec!["icd10"]),
            ("bodySite", vec!["bodySite"]),
            ("urn:oid:2.16.840.1.113883.6.43.1", vec!["lokalisation_icd_o_3"]),
            ("59542-1", vec!["loinc", "gradingcs"]),
            ("metastases_present", vec!["loinc", "jnucs"]),
            ("localization_metastases", vec!["loinc", "fmlokalisationcs"]),
            ("59847-4", vec!["loinc", "morph"]),
            ("sample_kind", vec!["specimentype"]),
            ("21908-9", vec!["loinc", "uiccstadiumcs"]),
            ("21905-5", vec!["loinc", "TNMTCS"]),
            ("21906-3", vec!["loinc", "TNMNCS"]),
            ("21907-1", vec!["loinc", "TNMMCS"]),
            ("42030-7", vec!["loinc", "TNMmSymbolCS"]),
            ("59479-6", vec!["loinc", "TNMySymbolCS"]),
            ("21983-2", vec!["loinc", "TNMrSymbolCS"]),
            ("21899-0", vec!["loinc", "TNMTCS"]),
            ("21900-6", vec!["loinc", "TNMNCS"]),
            ("21901-4", vec!["loinc", "TNMMCS"]),
            ("42030-7", vec!["loinc", "TNMmSymbolCS"]),
            ("59479-6", vec!["loinc", "TNMySymbolCS"]),
            ("21983-2", vec!["loinc", "TNMrSymbolCS"]),
            ("48018-6", vec!["loinc", "molecularMarker"]),
            ("local_assessment_residual_tumor", vec!["Therapieart", "lokalebeurteilungresidualstatuscs"]),
            ("21976-6", vec!["loinc", "gesamtbeurteilungtumorstatuscs"]),
            ("LA4583-6", vec!["loinc", "verlauflokalertumorstatuscs"]),
            ("LA4370-8", vec!["loinc", "verlauftumorstatuslymphknotencs"]),
            ("LA4226-2", vec!["loinc", "verlauftumorstatusfernmetastasencs"]),
            ("75186-7", vec!["loinc", "vitalstatuscs"]),
        ])
    }

    fn append_cql_snippets(&self, map: &mut HashMap<(&str, CriterionRole), &str>) {
        for key in ["59542-1", "59847-4", "21976-6", "LA4583-6", "LA4370-8", "LA4226-2", "75186-7"] { // observation
            // TODO: Revert to first expression if https://github.com/samply/blaze/issues/808 is solved
            map.insert((key, CriterionRole::Query), "exists from (Observation: Code '{{K}}' from {{A1}}) O\nwhere O.value.coding contains Code '{{C}}' from {{A2}}");
            map.insert((key, CriterionRole::Query), "exists from (Observation: Code '{{K}}' from {{A1}}) O\nwhere O.value.coding.code contains '{{C}}'");
        }
        for key in ["OP", "ST"] { // procedure
            map.insert((key, CriterionRole::Query), "exists (Procedure: category in Code '{{K}}' from {{A1}})");
        }
        for key in ["CH", "HO", "IM", "KM"] { // medicationStatement
            map.insert((key, CriterionRole::Query), "exists (MedicationStatement: category in Code '{{K}}' from {{A1}})");
        }
        for key in ["21905-5", "21906-3", "21907-1", "42030-7", "59479-6", "21983-2"] { // TNMc
            map.insert((key, CriterionRole::Query), "exists from (Observation: Code '21908-9' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}");
        }
        for key in ["21899-0", "21900-6", "21901-4", "42030-7", "59479-6", "21983-2"] { // TNMp
            map.insert((key, CriterionRole::Query), "exists from (Observation: Code '21902-2' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}");
        }


        map.extend( [
            (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
            (
                ("pseudo_projects", CriterionRole::Query),
                "  exists ( Patient.extension E where E.url = 'http://dktk.dkfz.de/fhir/projects/{{C}}')",
            ),
            (("diagnosis",CriterionRole::Query), "exists (Condition: Code '{{C}}' from {{A1}})"),
            (
                ("bodySite",CriterionRole::Query),
                "exists from (Condition) C\nwhere C.bodySite.coding contains Code '{{C}}' from {{A1}}",
            ),
            //TODO Revert to first expression if https://github.com/samply/blaze/issues/808 is solved
            // (("urn:oid:2.16.840.1.113883.6.43.1", CriterionRole::Query), "exists from (Condition) C\nwhere C.bodySite.coding contains Code '{{C}}' from {{A1}}"),
            (
                ("urn:oid:2.16.840.1.113883.6.43.1", CriterionRole::Query),
                "exists from (Condition) C\nwhere C.bodySite.coding.code contains '{{C}}'",
            ),
            (
                ("year_of_diagnosis", CriterionRole::Query),
                "exists from (Condition) C\nwhere year from C.onset between {{D1}} and {{D2}}",
            ),
            (
                ("conditionLowerThanDate", CriterionRole::Query),
                "exists from (Condition) C\nwhere year from C.onset <= {{D2}}",
            ),
            (
                ("conditionGreaterThanDate", CriterionRole::Query),
                "exists from (Condition) C\nwhere year from C.onset >= {{D1}}",
            ),
            (
                ("age_at_diagnosis", CriterionRole::Query),
                "exists (Condition) C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between {{D1}} and {{D2}}",
            ),
            (
                ("conditionLowerThanAge", CriterionRole::Query),
                "exists (Condition) C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) <= {{D2}}",
            ),
            (
                ("conditionGreaterThanAge", CriterionRole::Query),
                "exists (Condition) C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) >= {{D1}}",
            ),
            (
                ("year_of_primary_diagnosis", CriterionRole::Query),
                "year from PrimaryDiagnosis.onset between {{D1}} and {{D2}}",
            ),
            (
                ("primaryConditionLowerThanDate", CriterionRole::Query),
                "year from PrimaryDiagnosis.onset <= {{D2}}",
            ),
            (
                ("primaryConditionGreaterThanDate", CriterionRole::Query),
                "year from PrimaryDiagnosis.onset >= {{D1}}",
            ),
            (
                ("age_at_primary_diagnosis", CriterionRole::Query),
                "AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) between {{D1}} and {{D2}}",
            ),
            (
                ("primaryConditionLowerThanAge", CriterionRole::Query),
                "AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) <= {{D2}}",
            ),
            (
                ("primaryConditionGreaterThanAge", CriterionRole::Query),
                "AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) >= {{D1}}",
            ),
            (
                ("metastases_present", CriterionRole::Query),
                "exists from (Observation: Code '21907-1' from {{A1}}) O\nwhere O.value.coding.code contains '{{C}}'",
            ),
            (
                ("localization_metastases", CriterionRole::Query),
                "exists from (Observation: Code '21907-1' from {{A1}}) O\nwhere O.bodySite.coding.code contains '{{C}}'",
            ),
            (
                ("48018-6", CriterionRole::Query),
                "exists from (Observation: Code '69548-6' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}",
            ),
            (
                ("48005-3", CriterionRole::Query),
                "exists from (Observation: Code '69548-6' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ), //TODO @ThomasK replace C with S
            (
                ("81290-9", CriterionRole::Query),
                "exists from (Observation: Code '69548-6' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("81248-7", CriterionRole::Query),
                "exists from (Observation: Code '69548-6' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("81249-5", CriterionRole::Query),
                "exists from (Observation: Code '69548-6' from {{A1}}) O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("local_assessment_residual_tumor", CriterionRole::Query),
                "exists from (Procedure: category in Code 'OP' from {{A1}}) P\nwhere P.outcome.coding.code contains '{{C}}'",
            ),
            (("pat_with_samples", CriterionRole::Query), "exists (Specimen)"),
            (("sample_kind", CriterionRole::Query), "exists (Specimen: Code '{{C}}' from {{A1}})"),
            (("retrieveSpecimenByType", CriterionRole::Query), "(S.type.coding.code contains '{{C}}')"),
            (
                ("Organization", CriterionRole::Query),
                "Patient.managingOrganization.reference = \"Organization Ref\"('Klinisches Krebsregister/ITM')",
            ),
            (
                ("department", CriterionRole::Query),
                "exists from (Encounter) I\nwhere I.identifier.value = '{{C}}' ",
            ),
            (
                ("21908-9", CriterionRole::Query),
                "(exists ((Observation: Code '21908-9' from loinc) O where O.value.coding.code contains '{{C}}')) or (exists ((Observation: Code '21902-2' from loinc) O where O.value.coding.code contains '{{C}}'))",
            ),
            (("histology", CriterionRole::Query),"exists from (Observation: Code '59847-4' from loinc) O\n"),
        ]);
    }

    fn append_mandatory_code_lists(&self, _map: &mut IndexSet<&str>) {
        //let mut set = map.remove(self.name()).unwrap_or(IndexSet::new());
        // for value in ["icd10", "SampleMaterialType", "loinc"] {
        //     map.insert(value);
        // }
        //map.insert(self.name(), set);
    }

    fn append_cql_template(&self, _template: &mut String) {
        //include_str!("template.cql")
    }

    fn name(&self) -> &'static ProjectName {
        &ProjectName::Dktk
    }

    fn append_body(&self, str: &mut String) {
        str.push_str(include_str!("body.json"));
    }

    fn append_sample_type_workarounds(&self, _map: &mut HashMap<&str, Vec<&str>>) {
        //none
    }
}
