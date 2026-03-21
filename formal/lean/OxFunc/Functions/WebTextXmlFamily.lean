import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def encodeurlMeta : FunctionMeta := {
  functionId := "FUNC.ENCODEURL"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def filterxmlMeta : FunctionMeta := {
  functionId := "FUNC.FILTERXML"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

inductive FilterXmlAdmittedResult where
  | emptyNodeSet
  | unsupportedXPathResult
  | nodeStrings (items : List String)
  deriving Repr, DecidableEq

inductive WebTextXmlProjected where
  | worksheetError (code : WorksheetErrorCode)
  | scalarText (text : String)
  | verticalTextArray (items : List String)
  deriving Repr, DecidableEq

def filterxmlAdmittedProjection : FilterXmlAdmittedResult → WebTextXmlProjected
  | .emptyNodeSet => .worksheetError OxFunc.WorksheetErrorCode.value
  | .unsupportedXPathResult => .worksheetError OxFunc.WorksheetErrorCode.value
  | .nodeStrings [] => .worksheetError OxFunc.WorksheetErrorCode.value
  | .nodeStrings [item] => .scalarText item
  | .nodeStrings items => .verticalTextArray items

theorem web_text_xml_meta_shapes :
    encodeurlMeta.arity = Arity.exact 1
    ∧ encodeurlMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ filterxmlMeta.arity = Arity.exact 2
    ∧ filterxmlMeta.hostInteraction = HostInteractionClass.none := by
  simp [encodeurlMeta, filterxmlMeta]

theorem filterxml_empty_nodeset_is_value_error :
    filterxmlAdmittedProjection .emptyNodeSet =
      WebTextXmlProjected.worksheetError OxFunc.WorksheetErrorCode.value := by
  rfl

theorem filterxml_unsupported_xpath_result_is_value_error :
    filterxmlAdmittedProjection .unsupportedXPathResult =
      WebTextXmlProjected.worksheetError OxFunc.WorksheetErrorCode.value := by
  rfl

theorem filterxml_multiple_nodes_project_to_vertical_array :
    filterxmlAdmittedProjection (.nodeStrings ["1", "2"]) =
      WebTextXmlProjected.verticalTextArray ["1", "2"] := by
  simp [filterxmlAdmittedProjection]

end OxFunc.Functions
