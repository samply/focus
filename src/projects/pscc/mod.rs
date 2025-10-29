use std::{collections::HashMap, sync::LazyLock};

use indexmap::IndexSet;

use super::CriterionRole;

pub static CODE_LISTS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("icd10", "http://hl7.org/fhir/sid/icd-10"),
        ("loinc", "http://loinc.org"),

        //morph
        ("icd-o-3-morphology", "http://terminology.hl7.org/ValueSet/icd-o-3"),
        //lokalisation_icd_o_3
        ("icd-O3-topography", "http://hl7.org/fhir/sid/icd-O3-topography"),
        (//TODO
            "Therapieart",
            "https://simplifier.net/PSCC-Test/StructureDefinition/SystemicTherapy",
        ),
        (//TODO - TNM
            "uiccstadiumcs",
            "https://simplifier.net/PSCC/tnmstagevs",
        ),
        (//TODO
            "vitalstatuscs",
            "http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VitalstatusCS",
        ),
        ("molecularMarker", "http://www.genenames.org"),//TODO
    ])
});

pub static OBSERVATION_LOINC_CODES: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            ("grading", "59542-1"),
            ("morphology", "59847-4"),
            ("responseOverTime", "21976-6"),
            ("localRegionalRecurrence", "LA4583-6"),
            ("lymphNodeRecurrence", "LA4370-8"),
            ("distantMetastases", "LA4226-2"),
            ("vitalStatus", "75186-7"),
            ("observationMolecularMarkerName", "48018-6"),
            ("observationMolecularMarkerAminoacidchange", "48005-3"),
            ("observationMolecularMarkerDNAchange", "81290-9"),
            ("observationMolecularMarkerSeqRefNCBI", "81248-7"),
            ("observationMolecularMarkerEnsemblID", "81249-5"),
        ])
    });

pub static CRITERION_CODE_LISTS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("diagnosis", vec!["icd10"]),
            ("bodySite", vec!["bodySite"]),
            ("conditionLocalization", vec!["lokalisation_icd_o_3"]),
            ("grading", vec!["loinc", "gradingcs"]),
            ("metastases_present", vec!["loinc", "jnucs"]),
            ("localization_metastases", vec!["loinc", "fmlokalisationcs"]),
            ("procedure", vec!["Therapieart"]),
            ("medicationStatement", vec!["Therapieart"]),
            ("morphology", vec!["loinc", "morph"]),
            ("sample_kind", vec!["specimentype"]),
            (
                "observationMolecularMarkerName",
                vec!["loinc", "molecularMarker"],
            ),
            ("observationMolecularMarkerAminoacidchange", vec!["loinc"]),
            ("observationMolecularMarkerDNAchange", vec!["loinc"]),
            ("observationMolecularMarkerSeqRefNCBI", vec!["loinc"]),
            ("observationMolecularMarkerEnsemblID", vec!["loinc"]),
            (
                "local_assessment_residual_tumor",
                vec!["Therapieart", "lokalebeurteilungresidualstatuscs"],
            ),
            (
                "responseOverTime",
                vec!["loinc", "gesamtbeurteilungtumorstatuscs"],
            ),
            (
                "localRegionalRecurrence",
                vec!["loinc", "verlauflokalertumorstatuscs"],
            ),
            (
                "lymphNodeRecurrence",
                vec!["loinc", "verlauftumorstatuslymphknotencs"],
            ),
            (
                "distantMetastases",
                vec!["loinc", "verlauftumorstatusfernmetastasencs"],
            ),
            ("vitalStatus", vec!["loinc", "vitalstatuscs"]),
            ("TNM-T", vec!["loinc", "TNMTCS"]),
            ("TNM-N", vec!["loinc", "TNMNCS"]),
            ("TNM-M", vec!["loinc", "TNMMCS"]),
            ("TNM-m-Symbol", vec!["loinc", "TNMmSymbolCS"]),
            ("TNM-y-Symbol", vec!["loinc", "TNMySymbolCS"]),
            ("TNM-r-Symbol", vec!["loinc", "TNMrSymbolCS"]),
            ("chemo-hki", vec!["Therapieart"]), // search therapies with specific therapyline
            ("immun-hki", vec!["Therapieart"]),
            ("targeted-therapy-hki", vec!["Therapieart"]),
        ])
    });

pub static CQL_SNIPPETS: LazyLock<HashMap<(&'static str, CriterionRole), &'static str>> =
    LazyLock::new(|| {
        let observation = "exists from [Observation: Code '{{K}}' from {{A1}}] O\nwhere O.value.coding.code contains '{{C}}'";

        HashMap::from([
        (
            ("gender", CriterionRole::Query),
            "Patient.gender = '{{C}}'",
        ),
        (
            ("pseudo_projects", CriterionRole::Query),
            "exists ( Patient.extension E where E.url = 'http://dktk.dkfz.de/fhir/projects/{{C}}')",
        ),
        (
            ("diagnosis", CriterionRole::Query),
            "exists [Condition: Code '{{C}}' from {{A1}}]",
        ),
        (
            ("bodySite", CriterionRole::Query),
            "exists from [Condition] C\nwhere C.bodySite.coding contains Code '{{C}}' from {{A1}}",
        ),
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
            ("vitalStatus", CriterionRole::Query),
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
        (
            ("sample_kind", CriterionRole::Query),
            "exists [Specimen: Code '{{C}}' from {{A1}}]",
        ),
        (
            ("sample_kind", CriterionRole::Filter),
            "(S.type.coding.code contains '{{C}}')",
        ),
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
        (
            ("histology", CriterionRole::Query),
            "exists from [Observation: Code '59847-4' from loinc] O\n",
        ),
        (
            ("chemo-hki", CriterionRole::Query),
            "exists [MedicationStatement: category in Code 'CH' from {{A1}}] M\nwhere M.extension.where(url='http://hki.de/fhir/StructureDefinition/Therapielinie').value.text = '{{C}}'",
        ),
        (
            ("immun-hki", CriterionRole::Query),
            "exists [MedicationStatement: category in Code 'IM' from {{A1}}] M\nwhere M.extension.where(url='http://hki.de/fhir/StructureDefinition/Therapielinie').value.text = '{{C}}'",
        ),
        (
            ("targeted-therapy-hki", CriterionRole::Query),
            "exists [MedicationStatement: category in Code 'ZS' from {{A1}}] M\nwhere M.extension.where(url='http://hki.de/fhir/StructureDefinition/Therapielinie').value.text = '{{C}}'",
        ),
        (
            ("therapy-intention-hki", CriterionRole::Query),
            "exists [MedicationStatement] M\nwhere M.extension.where(url='http://dktk.dkfz.de/fhir/StructureDefinition/onco-core-Extension-SYSTIntention').value.coding.code = '{{C}}'",
        ),
        (
            ("consent-hki", CriterionRole::Query),
            "exists [Consent]"
        ),
    ])
    });

pub static MANDATORY_CODE_LISTS: LazyLock<IndexSet<&'static str>> =
    LazyLock::new(|| IndexSet::from(["loinc"]));

pub static SAMPLE_TYPE_WORKAROUNDS: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::new() // No workarounds for dhki
    });
