library Retrieve
using FHIR version '4.0.0'
include FHIRHelpers version '4.0.0'

codesystem icd10: 'http://hl7.org/fhir/sid/icd-10'
codesystem SampleMaterialType: 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType'
codesystem icd10gm: 'http://fhir.de/CodeSystem/dimdi/icd-10-gm'
codesystem icd10gmnew: 'http://fhir.de/CodeSystem/bfarm/icd-10-gm'
codesystem loinc: 'http://loinc.org'
codesystem FastingStatus: 'http://terminology.hl7.org/CodeSystem/v2-0916'
codesystem StorageTemperature: 'https://fhir.bbmri.de/CodeSystem/StorageTemperature'


context Patient

define AgeClass:
if (Patient.birthDate is null) then 'unknown' else ToString((AgeInYears() div 10) * 10)

define Gender:
if (Patient.gender is null) then 'unknown' else Patient.gender

define Custodian:
    First(from Specimen.extension E
    where E.url = 'https://fhir.bbmri.de/StructureDefinition/Custodian'
    return (E.value as Reference).identifier.value)

define function SampleType(specimen FHIR.Specimen):
    case FHIRHelpers.ToCode(specimen.type.coding.where(system = 'https://fhir.bbmri.de/CodeSystem/SampleMaterialType').first())
       when Code 'plasma-edta' from SampleMaterialType then 'blood-plasma'
       when Code 'plasma-citrat' from SampleMaterialType then 'blood-plasma'
       when Code 'plasma-heparin' from SampleMaterialType then 'blood-plasma'
       when Code 'plasma-cell-free' from SampleMaterialType then 'blood-plasma'
       when Code 'plasma-other' from SampleMaterialType then 'blood-plasma'
       when Code 'plasma' from SampleMaterialType then 'blood-plasma'
       when Code 'tissue-formalin' from SampleMaterialType then 'tissue-ffpe'
       when Code 'tumor-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe'
       when Code 'normal-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe'
       when Code 'other-tissue-ffpe' from SampleMaterialType then 'tissue-ffpe'
       when Code 'tumor-tissue-frozen' from SampleMaterialType then 'tissue-frozen'
       when Code 'normal-tissue-frozen' from SampleMaterialType then 'tissue-frozen'
       when Code 'other-tissue-frozen' from SampleMaterialType then 'tissue-frozen'
       when Code 'tissue-paxgene-or-else' from SampleMaterialType then 'tissue-other'
       when Code 'derivative' from SampleMaterialType then 'derivative-other'
       when Code 'liquid' from SampleMaterialType then 'liquid-other'
       when Code 'tissue' from SampleMaterialType then 'tissue-other'
       when Code 'serum' from SampleMaterialType then 'blood-serum'
       when Code 'cf-dna' from SampleMaterialType then 'dna'
       when Code 'g-dna' from SampleMaterialType then 'dna'
       when Code 'blood-plasma' from SampleMaterialType then 'blood-plasma'
       when Code 'tissue-ffpe' from SampleMaterialType then 'tissue-ffpe'
       when Code 'tissue-frozen' from SampleMaterialType then 'tissue-frozen'
       when Code 'tissue-other' from SampleMaterialType then 'tissue-other'
       when Code 'derivative-other' from SampleMaterialType then 'derivative-other'
       when Code 'liquid-other' from SampleMaterialType then 'liquid-other'
       when Code 'blood-serum' from SampleMaterialType then 'blood-serum'
       when Code 'dna' from SampleMaterialType then 'dna'
       when Code 'buffy-coat' from SampleMaterialType then 'buffy-coat'
       when Code 'urine' from SampleMaterialType then 'urine'
       when Code 'ascites' from SampleMaterialType then 'ascites'
       when Code 'saliva' from SampleMaterialType then 'saliva'
       when Code 'csf-liquor' from SampleMaterialType then 'csf-liquor'
       when Code 'bone-marrow' from SampleMaterialType then 'bone-marrow'
       when Code 'peripheral-blood-cells-vital' from SampleMaterialType then 'peripheral-blood-cells-vital'
       when Code 'stool-faeces' from SampleMaterialType then 'stool-faeces'
       when Code 'rna' from SampleMaterialType then 'rna'
       when Code 'whole-blood' from SampleMaterialType then 'whole-blood'
       when Code 'swab' from SampleMaterialType then 'swab'
       when Code 'dried-whole-blood' from SampleMaterialType then 'dried-whole-blood'
       when null  then 'Unknown'
       else 'Unknown'
   end
define Specimen:
    if InInitialPopulation then [Specimen] S where (((((S.collection.fastingStatus.coding.code contains 'Sober') ) or ((S.collection.fastingStatus.coding.code contains 'Not sober') ))) or (((((( (S.type.coding.code contains 'blood-serum')))) or ((( (S.type.coding.code contains 'serum')))))) or ((((( (S.type.coding.code contains 'tissue-other')))) or ((( (S.type.coding.code contains 'tissue-paxgene-or-else')))) or ((( (S.type.coding.code contains 'tissue'))))))) or ((FHIRHelpers.ToDateTime(S.collection.collected) between @2023-10-29 and @2023-10-30) ) or ((((S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains 'temperature2to10')) or ((S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding.code contains 'temperatureGN'))))) else {} as List<Specimen>

define Diagnosis:
if InInitialPopulation then [Condition] else {} as List<Condition>

define function DiagnosisCode(condition FHIR.Condition):
condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first()

define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):
Coalesce(
  condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first(),
  condition.code.coding.where(system = 'http://fhir.de/CodeSystem/dimdi/icd-10-gm').code.first(),
  condition.code.coding.where(system = 'http://fhir.de/CodeSystem/bfarm/icd-10-gm').code.first(),
  specimen.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code.first()
  )

define InInitialPopulation:
((((Patient.gender = 'other') or (Patient.gender = 'male'))) and (((((exists[Condition: Code 'C24' from icd10]) or (exists[Condition: Code 'C24' from icd10gm]) or (exists[Condition: Code 'C24' from icd10gmnew])) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/SampleDiagnosis').value.coding.code contains 'C24'))))) and (exists from [Condition] C
where AgeInYearsAt(FHIRHelpers.ToDateTime(C.onset)) between Ceiling(1) and Ceiling(11)) and (exists from [Condition] C
where FHIRHelpers.ToDateTime(C.onset) between @2023-10-29 and @2023-10-30) and (exists from [Observation: Code '39156-5' from loinc] O
where ((O.value as Quantity) < 1 'kg/m2' and (O.value as Quantity) > 111 'kg/m2')) and (exists from [Observation: Code '29463-7' from loinc] O
where ((O.value as Quantity) < 110 'kg' and (O.value as Quantity) > 1111 'kg')) and (((exists from [Specimen] S
where S.collection.fastingStatus.coding.code contains 'Sober' ) or (exists from [Specimen] S
where S.collection.fastingStatus.coding.code contains 'Not sober' ))) and (((exists from [Observation: Code '72166-2' from loinc] O
where O.value.coding.code contains 'Smoker' ) or (exists from [Observation: Code '72166-2' from loinc] O
where O.value.coding.code contains 'Never smoked' ))) and ( AgeInYears() between Ceiling(1) and Ceiling(123)) and (((((( exists [Specimen: Code 'blood-serum' from SampleMaterialType]))) or ((( exists [Specimen: Code 'serum' from SampleMaterialType]))))) or ((((( exists [Specimen: Code 'tissue-other' from SampleMaterialType]))) or ((( exists [Specimen: Code 'tissue-paxgene-or-else' from SampleMaterialType]))) or ((( exists [Specimen: Code 'tissue' from SampleMaterialType])))))) and (exists from [Specimen] S
where FHIRHelpers.ToDateTime(S.collection.collected) between @2023-10-29 and @2023-10-30 ) and (((exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code 'temperature2to10' from StorageTemperature) ) or (exists from [Specimen] S where (S.extension.where(url='https://fhir.bbmri.de/StructureDefinition/StorageTemperature').value.coding contains Code 'temperatureGN' from StorageTemperature) ))))