library Retrieve
using FHIR version '4.0.0'
include FHIRHelpers version '4.0.0'

{{lists}}

context Patient

define Gender:
if (Patient.gender is null) then 'unknown' else Patient.gender

define AgeClass:
ToString((AgeInYears() div 10) * 10)

define PatientDeceased:
First (from [Observation: Code '75186-7' from loinc] O return O.value.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/VitalstatusCS').code.first())
define Deceased:
if (PatientDeceased is null) then 'unbekannt' else PatientDeceased

define Diagnosis:
if InInitialPopulation then [Condition] else {} as List<Condition>
define function DiagnosisCode(condition FHIR.Condition):
condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first()

define Specimen:
if InInitialPopulation then [Specimen] else {} as List<Specimen>
define function SampleType(specimen FHIR.Specimen):
specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').code.first()
define function SampleSubtype(specimen FHIR.Specimen):
specimen.type.text.first()

define Procedure:
if InInitialPopulation then [Procedure] else {} as List <Procedure>
define function ProcedureType(procedure FHIR.Procedure):
procedure.category.coding.where(system = 'http://dktk.dkfz.de/fhir/onco/core/CodeSystem/SYSTTherapieartCS').code.first()

define MedicationStatement:
if InInitialPopulation then [MedicationStatement] else {} as List <MedicationStatement>
define function AppliedMedications(medication FHIR.MedicationStatement):
medication.medication.coding.code.last()

define Encounter:
if InInitialPopulation then [Encounter] else {} as List<Encounter>
define function Departments(encounter FHIR.Encounter):
encounter.identifier.where(system = 'http://dktk.dkfz.de/fhir/sid/hki-department').value.first()

define InInitialPopulation:
{{retrieval_criteria}}
