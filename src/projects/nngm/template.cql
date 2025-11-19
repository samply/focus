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

define FirstDiagnosis:
First(
from [Condition] C
sort by date from onset asc)

define AgeClass:
if (PrimaryDiagnosis.onset is null)
then ToString((AgeInYearsAt(FHIRHelpers.ToDateTime(FirstDiagnosis.onset)) div 10) * 10)
else ToString((AgeInYearsAt(FHIRHelpers.ToDateTime(PrimaryDiagnosis.onset)) div 10) * 10)

define PatientDeceased:
First (from [Observation: Code '67162-8' from loinc] O return O.value.coding.where(system = 'http://uk-koeln.de/fhir/CodeSystem/nNGM/Vitalstatus').code.first())

define Deceased:
if (PatientDeceased is null) then 'unknown' else PatientDeceased

define SmokingHabit:
First (from [Observation: Code '72166-2' from loinc] O return O.value.coding.where(system = 'http://loinc.org').code.first())

define smoking:
if (SmokingHabit is null) then 'unknown' else SmokingHabit

define ecogStatus:
First (from [Observation: Code '89247-1' from loinc] O return O.value.coding.where(system = 'http://loinc.org').code.first())

define ecog:
if (ecogStatus is null) then 'unknown' else ecogStatus

define Diagnosis:
if InInitialPopulation then [Condition] else {} as List<Condition>

define Specimen:
if InInitialPopulation then [Specimen] else {} as List<Specimen>

define function SampleType(specimen FHIR.Specimen):
specimen.type.coding.where(system = 'https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SampleMaterialType').code.first()

define Procedure:
if InInitialPopulation then [Procedure] else {} as List <Procedure>

define function ProcedureType(procedure FHIR.Procedure):
procedure.category.coding.where(system = 'https://www.cancercoreeurope.eu/fhir/core/CodeSystem/SYSTTherapyTypeCS').code.first()

define MedicationStatement:
if InInitialPopulation then [MedicationStatement] else {} as List <MedicationStatement>

define function DiagnosisCode(condition FHIR.Condition):
condition.bodySite.coding.where(system = 'http://terminology.hl7.org/CodeSystem/icd-o-3').code.first()

define InInitialPopulation:
{{retrieval_criteria}}
