library Retrieve
using FHIR version '4.0.0'
include FHIRHelpers version '4.0.0'

codesystem loinc: 'http://loinc.org'
codesystem icd10: 'http://fhir.de/CodeSystem/bfarm/icd-10-gm'


context Patient

define Gender:
if (Patient.gender is null) then 'unknown' else Patient.gender

define PrimaryDiagnosis:
First(
from [Condition] C
where C.extension.where(url='http://hl7.org/fhir/StructureDefinition/condition-related').empty())

define AgeClass:
if (PrimaryDiagnosis.onset is null) then 'unknown' else ToString((AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) div 10) * 10)

define PatientDeceased:
First (from [Observation: Code '75186-7' from loinc] O return O.value.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VitalstatusCS').code.first())

define Deceased:
if (PatientDeceased is null) then 'unbekannt' else PatientDeceased

define Diagnosis:
if InInitialPopulation then [Condition] else {} as List<Condition>

define function DiagnosisCode(condition FHIR.Condition):
condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first()

define function SampleType(specimen FHIR.Specimen):
specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()

define Specimen:
if InInitialPopulation then [Specimen] S  else {} as List<Specimen>

define Procedure:
if InInitialPopulation then [Procedure] else {} as List <Procedure>

define function ProcedureType(procedure FHIR.Procedure):
procedure.category.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SYSTTherapieartCS').code.first()

define MedicationStatement:
if InInitialPopulation then [MedicationStatement] else {} as List <MedicationStatement>

define GeneticVariantCount:
if InInitialPopulation then [Observation: Code '69548-6' from loinc] else {} as List <Observation>

define function GeneticVariantCode(observation FHIR.Observation):
observation.component.where(code.coding contains Code '48018-6' from loinc).value.coding.code.first()

define InInitialPopulation:
(((((((exists [Condition: Code 'C30' from icd10]))) or (((exists [Condition: Code 'C30.1' from icd10]))) or (((exists [Condition: Code 'C30.0' from icd10])))))))
