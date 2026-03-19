library Retrieve
using FHIR version '4.0.1'
include FHIRHelpers version '4.0.1'

{{lists}}

context Patient

define AgeClass:
if (Patient.birthDate is null) then 'unknown' else ToString((AgeInYears() div 10) * 10)

define Gender:
if (Patient.gender is null) then 'unknown' else Patient.gender

define Custodian:
    First(
        from Specimen.extension E
        where E.url = 'https://fhir.bbmri-eric.eu/StructureDefinition/miabis-sample-collection-extension'
        return (E.value as Identifier).value
    )

define function SampleType(specimen FHIR.Specimen):
    case FHIRHelpers.ToCode(specimen.type.coding.where(system = 'https://fhir.bbmri-eric.eu/CodeSystem/miabis-detailed-samply-type-cs').first())
       when Code 'WholeBlood'         from MiabisDetailedSampleType then 'whole-blood'
       when Code 'BuffyCoat'          from MiabisDetailedSampleType then 'buffy-coat'
       when Code 'Plasma'             from MiabisDetailedSampleType then 'blood-plasma'
       when Code 'Serum'              from MiabisDetailedSampleType then 'blood-serum'
       when Code 'DNA'                from MiabisDetailedSampleType then 'dna'
       when Code 'RNA'                from MiabisDetailedSampleType then 'rna'
       when Code 'PBMC'               from MiabisDetailedSampleType then 'peripheral-blood-cells-vital'
       when Code 'TissueFreshFrozen'  from MiabisDetailedSampleType then 'tissue-frozen'
       when Code 'TissueFixed'        from MiabisDetailedSampleType then 'tissue-ffpe'
       when Code 'BoneMarrowAspirate' from MiabisDetailedSampleType then 'bone-marrow'
       when Code 'CerebrospinalFluid' from MiabisDetailedSampleType then 'csf-liquor'
       when Code 'Urine'              from MiabisDetailedSampleType then 'urine'
       when Code 'Saliva'             from MiabisDetailedSampleType then 'saliva'
       when Code 'Faeces'             from MiabisDetailedSampleType then 'stool-faeces'
       when Code 'LiquidBiopsy'       from MiabisDetailedSampleType then 'liquid-other'
       when Code 'Sputum'             from MiabisDetailedSampleType then 'liquid-other'
       when null                      then 'Unknown'
       else                                'Unknown'
    end

define StorageTemperature:
    First(
        from Specimen.processing P
        return First(
            from P.extension E
            where E.url = 'https://fhir.bbmri-eric.eu/StructureDefinition/miabis-sample-storage-temperature-extension'
            return FHIRHelpers.ToString((E.value as CodeableConcept).coding.first().code)
        )
    )

define Specimen:
    if InInitialPopulation then [Specimen] S {{filter_criteria}} else {} as List<Specimen>

define Diagnosis:
    if InInitialPopulation then [Condition] else {} as List<Condition>

define function DiagnosisCode(condition FHIR.Condition, specimen FHIR.Specimen):
    condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first()

define function DiagnosisCode(condition FHIR.Condition):
    condition.code.coding.where(system = 'http://hl7.org/fhir/sid/icd-10').code.first()

define InInitialPopulation:
{{retrieval_criteria}}
