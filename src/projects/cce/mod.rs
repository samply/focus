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
        HashMap::from([
            (("gender", CriterionRole::Query), "Patient.gender = '{{C}}'"),
            (
                ("diagnosis", CriterionRole::Query),
                "((exists[Condition: Code '{{C}}' from {{A1}}]) or (exists[Condition: Code '{{C}}' from {{A2}}]) or (exists[Condition: Code '{{C}}' from {{A3}}])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains '{{C}}'))",
            ),
            (("diagnosis_old", CriterionRole::Query), " exists [Condition: Code '{{C}}' from {{A1}}]"),
            (
                ("date_of_diagnosis", CriterionRole::Query),
                "exists from [Condition] C\nwhere FHIRHelpers.ToDateTime(C.onset) between {{D1}} and {{D2}}",
            ),
            (
                ("diagnosis_age_donor", CriterionRole::Query),
                "exists from [Condition] C\nwhere AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling({{D1}}) and Ceiling({{D2}})",
            ),
            (("donor_age", CriterionRole::Query), " AgeInYears() between Ceiling({{D1}}) and Ceiling({{D2}})"),
            (
                ("observationRange", CriterionRole::Query),
                "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value between {{D1}} and {{D2}}",
            ),
            (
                ("body_weight", CriterionRole::Query),
                "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg' and (O.value as Quantity) > {{D2}} 'kg')",
            ),
            (
                ("bmi", CriterionRole::Query),
                "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere ((O.value as Quantity) < {{D1}} 'kg/m2' and (O.value as Quantity) > {{D2}} 'kg/m2')",
            ),
            (("sample_kind", CriterionRole::Query), " exists [Specimen: Code '{{C}}' from {{A1}}]"),
            (("sample_kind", CriterionRole::Filter), " (S.type.coding.code contains '{{C}}')"),
            (
                ("storage_temperature", CriterionRole::Filter),
                "(S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains '{{C}}')",
            ),
            (
                ("sampling_date", CriterionRole::Filter),
                "(FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}}) ",
            ),
            (
                ("fasting_status", CriterionRole::Filter),
                "(S.collection.fastingStatus.coding.code contains '{{C}}') ",
            ),
            (
                ("sampling_date", CriterionRole::Query),
                "exists from [Specimen] S\nwhere FHIRHelpers.ToDateTime(S.collection.collected) between {{D1}} and {{D2}} ",
            ),
            (
                ("fasting_status", CriterionRole::Query),
                "exists from [Specimen] S\nwhere S.collection.fastingStatus.coding.code contains '{{C}}' ",
            ),
            (
                ("storage_temperature", CriterionRole::Query),
                "exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code '{{C}}' from {{A1}}) ",
            ),
            (
                ("smoking_status", CriterionRole::Query),
                "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}' ",
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

pub static SAMPLE_TYPE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::new() // No workarounds for cce
    });
