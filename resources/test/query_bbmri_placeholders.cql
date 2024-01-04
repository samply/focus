library Retrieve
using FHIR version '4.0.0'
include FHIRHelpers version '4.0.0'

codesystem loinc: 'http://loinc.org'

codesystem icd10: 'http://hl7.org/fhir/sid/icd-10'

codesystem SampleMaterialType: 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType'

context Patient

BBMRI_STRAT_AGE_STRATIFIER

BBMRI_STRAT_GENDER_STRATIFIER

BBMRI_STRAT_CUSTODIAN_STRATIFIER

BBMRI_STRAT_SAMPLE_TYPE_STRATIFIER

BBMRI_STRAT_DEF_SPECIMEN
    if InInitialPopulation then [Specimen] else {} as List<Specimen>

BBMRI_STRAT_DIAGNOSIS_STRATIFIER

BBMRI_STRAT_DEF_IN_INITIAL_POPULATION
true