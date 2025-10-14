use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

const ICD_10: &str = "icd10";
// const ICD_10_GM: &str = "icd10gm";
// const ICD_10_GM_New: &str = "icd10gmnew";
const LOINC: &str = "loinc";
const GRADING_CS: &str = "gradingCS";
const OPS: &str = "ops";
const MORPH: &str = "morph";
const LOCALIZATION_ICD_O_3: &str = "localization_icd_o_3";
const BODY_SITE: &str = "bodySite";
const THERAPY_TYPE_CS: &str = "therapyTypeCS";
const SPECIMEN_TYPE: &str = "specimenType";
const UICC_STAGE_CS: &str = "uiccStageCS";
const LOCAL_ASSESSMENT_RESIDUAL_TUMOR_CS: &str = "localAssessmentResidualTumorCS";
const OVERALL_ASSESSMENT_RESIDUAL_TUMOR_CS: &str = "overallAssessmentResidualTumorCS";
const PROGRESSION_LOCAL_TUMOR_STATUS_CS: &str = "progressionLocalTumorStatusCS";
// const VERLAUF_TUMOR_STATUS_LYMPH_KNOTEN_CS: &str = "verlaufTumorstatusLymphknotenCS";
// const VERLAUF_TUMOR_STATUS_FERN_METASTASEN_CS: &str = "verlaufTumorstatusFernmetastasenCS";
const VITAL_STATUS_CS: &str = "vitalStatusCS";
const YNU_CS: &str = "YNUCS";
// const FM_LOKALISATION_CS: &str = "fmlokalisationcs";
const TNM_T_CS: &str = "TNMTCS";
const TNM_N_CS: &str = "TNMNCS";
const TNM_M_CS: &str = "TNMMCS";
const TNM_Y_SYMBOL_CS: &str = "TNMySymbolCS";
const TNM_R_SYMBOL_CS: &str = "TNMrSymbolCS";
const TNM_M_SYMBOL_CS: &str = "TNMmSymbolCS";
const MOLECULAR_MARKER: &str = "molecularMarker";

pub static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (ICD_10, "http://fhir.de/CodeSystem/bfarm/icd-10-gm"),
        // ("icd10gm", "http://fhir.de/CodeSystem/dimdi/icd-10-gm"),
        // ("icd10gmnew", "http://fhir.de/CodeSystem/bfarm/icd-10-gm"),
        (LOINC, "http://loinc.org"),
        (
            GRADING_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/GradingCS",
        ),
        (OPS, "http://fhir.de/CodeSystem/bfarm/ops"),
        (MORPH, "urn:oid:2.16.840.1.113883.6.43.1"),
        (LOCALIZATION_ICD_O_3, "urn:oid:2.16.840.1.113883.6.43.1"),
        (
            BODY_SITE,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TumorSiteLocationCS",
        ),
        (
            THERAPY_TYPE_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SYSTTherapyTypeCS",
        ),
        (
            SPECIMEN_TYPE,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SampleMaterialType",
        ),
        (
            UICC_STAGE_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/UICCStageCS",
        ),
        (
            LOCAL_ASSESSMENT_RESIDUAL_TUMOR_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/LocalAssessmentResidualTumorCS",
        ),
        (
            OVERALL_ASSESSMENT_RESIDUAL_TUMOR_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/OverallAssessmentResidualTumorCS",
        ),
        (
            PROGRESSION_LOCAL_TUMOR_STATUS_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/ProgressionLocalTumorStatusCS",
        ),
        // (
        //     "verlauftumorstatuslymphknotencs",
        //     "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VerlaufTumorstatusLymphknotenCS",
        // ),
        // (
        //     "verlauftumorstatusfernmetastasencs",
        //     "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VerlaufTumorstatusFernmetastasenCS",
        // ),
        (
            VITAL_STATUS_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VitalStatusCS",
        ),
        (
            YNU_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/YNUCS",
        ),
        // (
        //     "fmlokalisationcs",
        //     "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/FMLokalisationCS",
        // ),
        (
            TNM_T_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMTCS",
        ),
        (
            TNM_N_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMNCS",
        ),
        (
            TNM_M_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMMCS",
        ),
        (
            TNM_Y_SYMBOL_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMySymbolCS",
        ),
        (
            TNM_R_SYMBOL_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMrSymbolCS",
        ),
        (
            TNM_M_SYMBOL_CS,
            "https://www.cancercoreeurope.eu/fhir/core/CodeSystem/TNMmSymbolCS",
        ),
        (MOLECULAR_MARKER, "http://www.genenames.org"),
    ])
});

pub static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            (VITAL_STATUS_CS, "75186-7"),
            ("grading", "59542-1"),
            ("morphology", "59847-4"),
            ("body_weight", "29463-7"),
            ("bmi", "39156-5"),
            ("smoking_status", "72166-2"),
        ])
    });

pub static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            // TODO: add the code list for vital status?
            ("diagnosis", vec![ICD_10]),
            ("bodySite", vec![BODY_SITE]),
            ("conditionLocalization", vec![LOCALIZATION_ICD_O_3]),
            ("grading", vec![LOINC, GRADING_CS]),
            ("metastases_present", vec![LOINC, YNU_CS]),
            // ("localization_metastases", vec![LOINC, "fmlokalisationcs"]),
            ("procedure", vec![THERAPY_TYPE_CS]),
            ("medicationStatement", vec![THERAPY_TYPE_CS]),
            ("morphology", vec![LOINC, MORPH]),
            ("sample_kind", vec![SPECIMEN_TYPE]),
            (
                "observationMolecularMarkerName",
                vec![LOINC, MOLECULAR_MARKER],
            ),
            // ("observationMolecularMarkerAminoacidchange", vec![LOINC]),
            // ("observationMolecularMarkerDNAchange", vec![LOINC]),
            // ("observationMolecularMarkerSeqRefNCBI", vec![LOINC]),
            // ("observationMolecularMarkerEnsemblID", vec![LOINC]),
            (
                "local_assessment_residual_tumor",
                vec![THERAPY_TYPE_CS, LOCAL_ASSESSMENT_RESIDUAL_TUMOR_CS],
            ),
            (
                "responseOverTime",
                vec![LOINC, OVERALL_ASSESSMENT_RESIDUAL_TUMOR_CS],
            ),
            // (
            //     "localRegionalRecurrence",
            //     vec![LOINC, "verlauflokalertumorstatuscs"],
            // ),
            // (
            //     "lymphNodeRecurrence",
            //     vec![LOINC, "verlauftumorstatuslymphknotencs"],
            // ),
            // (
            //     "distantMetastases",
            //     vec![LOINC, "verlauftumorstatusfernmetastasencs"],
            // ),
            (VITAL_STATUS_CS, vec![LOINC, VITAL_STATUS_CS]),
            ("TNM-T", vec![LOINC, TNM_T_CS]),
            ("TNM-N", vec![LOINC, TNM_N_CS]),
            ("TNM-M", vec![LOINC, TNM_M_CS]),
            ("TNM-m-Symbol", vec![LOINC, TNM_M_SYMBOL_CS]),
            ("TNM-y-Symbol", vec![LOINC, TNM_Y_SYMBOL_CS]),
            ("TNM-r-Symbol", vec![LOINC, TNM_R_SYMBOL_CS]),
            // ("body_weight", vec!["loinc"]),
            // ("bmi", vec!["loinc"]),
            // ("smoking_status", vec!["loinc"]),
            // ("storage_temperature", vec!["StorageTemperature"]),
            // ("fasting_status", vec!["FastingStatus"]),
        ])
    });

pub static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> =
    LazyLock::new(|| {
        let observation = "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}'";

        HashMap::from([
            (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
            (
                ("diagnosis", CriterionRole::Query),
                "exists [Condition: Code '{{C}}' from {{A1}}]",
            ),
            (
                ("bodySite", CriterionRole::Query),
                "exists from [Condition] C\nwhere C.bodySite.coding contains Code '{{C}}' from {{A1}}",
            ),
            // TODO: Should we revert to first expression now that https://github.com/samply/blaze/issues/808 is solved?
            // ("conditionLocalization", "exists from [Condition] C\nwhere C.bodySite.coding contains Code '{{C}}' from {{A1}}"),
            (
                ("conditionLocalization", CriterionRole::Query),
                "exists from [Condition] C\nwhere C.bodySite.coding.code contains '{{C}}'",
            ),
            (
                ("year_of_diagnosis", CriterionRole::Query),
                "exists from [Condition] C\nwhere year from C.onset between {{D1}} and {{D2}}",
            ),
            (
                ("conditionLowerThanDate", CriterionRole::Query),
                "exists from [Condition] C\nwhere year from C.onset <= {{D2}}",
            ),
            (
                ("conditionGreaterThanDate", CriterionRole::Query),
                "exists from [Condition] C\nwhere year from C.onset >= {{D1}}",
            ),
            (
                ("age_at_diagnosis", CriterionRole::Query),
                "exists [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between {{D1}} and {{D2}}",
            ),
            (
                ("conditionLowerThanAge", CriterionRole::Query),
                "exists [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) <= {{D2}}",
            ),
            (
                ("conditionGreaterThanAge", CriterionRole::Query),
                "exists [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) >= {{D1}}",
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
                ("grading", CriterionRole::Query),
                observation,
            ),
            (
                ("morphology", CriterionRole::Query),
                observation,
            ),
            (
                ("responseOverTime", CriterionRole::Query),
                observation,
            ),
            (
                ("localRegionalRecurrence", CriterionRole::Query),
                observation,
            ),
            (
                ("lymphNodeRecurrence", CriterionRole::Query),
                observation,
            ),
            (
                ("distantMetastases", CriterionRole::Query),
                observation,
            ),
            (
                (VITAL_STATUS_CS, CriterionRole::Query),
                observation,
            ),
            (
                ("metastases_present", CriterionRole::Query),
                "exists from [Observation: Code '21907-1' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}'",
            ),
            (
                ("localization_metastases", CriterionRole::Query),
                "exists from [Observation: Code '21907-1' from {{A1}}] O\nwhere O.bodySite.coding.code contains '{{C}}'",
            ),
            (
                ("observationMolecularMarkerName", CriterionRole::Query),
                "exists from [Observation: Code '69548-6' from {{A1}}] O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}",
            ),
            (
                ("observationMolecularMarkerAminoacidchange", CriterionRole::Query),
                "exists from [Observation: Code '69548-6' from {{A1}}] O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("observationMolecularMarkerDNAchange", CriterionRole::Query),
                "exists from [Observation: Code '69548-6' from {{A1}}] O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("observationMolecularMarkerSeqRefNCBI", CriterionRole::Query),
                "exists from [Observation: Code '69548-6' from {{A1}}] O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("observationMolecularMarkerEnsemblID", CriterionRole::Query),
                "exists from [Observation: Code '69548-6' from {{A1}}] O\nwhere O.component.where(code.coding contains Code '{{K}}' from {{A1}}).value = '{{C}}'",
            ),
            (
                ("procedure", CriterionRole::Query),
                "exists [Procedure: category in Code '{{C}}' from {{A1}}]",
            ),
            (
                ("medicationStatement", CriterionRole::Query),
                "exists [MedicationStatement: category in Code '{{C}}' from {{A1}}]",
            ),
            (
                ("local_assessment_residual_tumor", CriterionRole::Query),
                "exists from [Procedure: category in Code 'OP' from {{A1}}] P\nwhere P.outcome.coding.code contains '{{C}}'",
            ),
            (("sample_kind", CriterionRole::Query), " exists [Specimen: Code '{{C}}' from {{A1}}]"),
            (("sample_kind", CriterionRole::Filter), " (S.type.coding.code contains '{{C}}')"),
            (
                ("retrieveSpecimenByType", CriterionRole::Query),
                "(S.type.coding.code contains '{{C}}')",
            ),
            (
                ("TNM-T", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21905-5' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21905-5' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21899-0' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21899-0' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("TNM-N", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21906-3' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21906-3' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21900-6' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21900-6' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("TNM-M", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21907-1' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21907-1' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21901-4' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21901-4' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("TNM-m-Symbol", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '42030-7' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '42030-7' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("TNM-y-Symbol", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '59479-6' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '59479-6' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("TNM-r-Symbol", CriterionRole::Query),
                "(exists from [Observation: Code '21908-9' from {{A1}}] O where O.component.where(code.coding contains Code '21983-2' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}}) or\n(exists from [Observation: Code '21902-2' from {{A1}}] O where O.component.where(code.coding contains Code '21983-2' from {{A1}}).value.coding contains Code '{{C}}' from {{A2}})",
            ),
            (
                ("Organization", CriterionRole::Query),
                "Patient.managingOrganization.reference = \"Organization Ref\"('Klinisches Krebsregister/ITM')",
            ),
            (
                ("department", CriterionRole::Query),
                "exists from [Encounter] I\nwhere I.identifier.value = '{{C}}' ",
            ),
            (
                ("uiccstadium", CriterionRole::Query),
                "(exists ([Observation: Code '21908-9' from loinc] O where O.value.coding.code contains '{{C}}')) or (exists ([Observation: Code '21902-2' from loinc] O where O.value.coding.code contains '{{C}}'))",
            ),
        ])
    });

pub static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> = LazyLock::new(|| {
    IndexSet::from([
        LOINC,
        ICD_10,
        SPECIMEN_TYPE,
        THERAPY_TYPE_CS,
        VITAL_STATUS_CS,
    ])
});
