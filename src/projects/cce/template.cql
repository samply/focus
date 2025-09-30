library Retrieve
using FHIR version '4.0.0'
include FHIRHelpers version '4.0.0'

{{lists}}

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
First (from [Observation: Code '75186-7' from loinc] O return O.value.coding.where(system = 'https://www.cancercoreeurope.eu/fhir/core/CodeSystem/VitalStatusCS').code.first())

define Deceased:
if (PatientDeceased is null) then 'unknown' else PatientDeceased

define Diagnosis:
if InInitialPopulation then [Condition] else {} as List<Condition>

define function DiagnosisCode(condition FHIR.Condition):
condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first()

define function SampleType(specimen FHIR.Specimen):
specimen.type.coding.where(system = 'https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SampleMaterialType').code.first()

define Specimen:
if InInitialPopulation then [Specimen] S {{filter_criteria}} else {} as List<Specimen>

define Procedure:
if InInitialPopulation then [Procedure] else {} as List <Procedure>

define function ProcedureType(procedure FHIR.Procedure):
procedure.category.coding.where(system = 'https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SYSTTherapyTypeCS').code.first()

define MedicationStatement:
if InInitialPopulation then [MedicationStatement] else {} as List <MedicationStatement>

define InInitialPopulation:
{{retrieval_criteria}}
