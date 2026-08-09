import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def besselConvertBaseMeta : FunctionMeta := {
  functionId := "FUNC.BESSEL_BASE"
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

def besseliMeta : FunctionMeta := {
  besselConvertBaseMeta with
  functionId := "FUNC.BESSELI"
}

def besseljMeta : FunctionMeta := {
  besselConvertBaseMeta with
  functionId := "FUNC.BESSELJ"
}

def besselkMeta : FunctionMeta := {
  besselConvertBaseMeta with
  functionId := "FUNC.BESSELK"
}

def besselyMeta : FunctionMeta := {
  besselConvertBaseMeta with
  functionId := "FUNC.BESSELY"
}

/--
W109 executable route tag for the two BESSELJ asymptotic seeds. Both sites
consume the shared worksheet-COS publication graph; only J0 stages the
published cosine times its P polynomial through the x87 double-rounded helper.
The tag captures that binding without duplicating the floating-point kernel.
-/
inductive BesselJAsymptoticSeed where
  | j0
  | j1
  deriving DecidableEq, Repr

inductive BesselJCosinePublicationRoute where
  | worksheetCosWithJ0X87Product
  | worksheetCosPlainProduct
  deriving DecidableEq, Repr

def besseljCosinePublicationRoute :
    BesselJAsymptoticSeed → BesselJCosinePublicationRoute
  | .j0 => .worksheetCosWithJ0X87Product
  | .j1 => .worksheetCosPlainProduct

def besselSeed (fnName : String) (x order : Rat) : Option Rat :=
  if fnName = "BESSELI" ∧ x = (3 / 2 : Rat) ∧ order = 1 then
    some ((981666428 : Rat) / 1000000000)
  else if fnName = "BESSELJ" ∧ x = (19 / 10 : Rat) ∧ order = 2 then
    some ((329925829 : Rat) / 1000000000)
  else if fnName = "BESSELK" ∧ x = (3 / 2 : Rat) ∧ order = 1 then
    some ((277387804 : Rat) / 1000000000)
  else if fnName = "BESSELY" ∧ x = (5 / 2 : Rat) ∧ order = 1 then
    some ((145918138 : Rat) / 1000000000)
  else
    none

theorem besselConvert_seed_rows :
    besselSeed "BESSELI" (3 / 2 : Rat) 1 = some ((981666428 : Rat) / 1000000000)
    ∧ besselSeed "BESSELJ" (19 / 10 : Rat) 2 = some ((329925829 : Rat) / 1000000000)
    ∧ besselSeed "BESSELK" (3 / 2 : Rat) 1 = some ((277387804 : Rat) / 1000000000)
    ∧ besselSeed "BESSELY" (5 / 2 : Rat) 1 = some ((145918138 : Rat) / 1000000000) := by
  native_decide

theorem besselConvert_profiles :
    besseliMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ besseljMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ besselkMeta.hostInteraction = HostInteractionClass.none
    ∧ besselyMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [besselConvertBaseMeta, besseliMeta, besseljMeta, besselkMeta, besselyMeta]

theorem besselConvert_ids :
    besseliMeta.functionId = "FUNC.BESSELI"
    ∧ besseljMeta.functionId = "FUNC.BESSELJ"
    ∧ besselkMeta.functionId = "FUNC.BESSELK"
    ∧ besselyMeta.functionId = "FUNC.BESSELY" := by
  simp [besselConvertBaseMeta, besseliMeta, besseljMeta, besselkMeta, besselyMeta]

theorem besselj_asymptotic_cosine_routes_are_site_specific :
    besseljCosinePublicationRoute .j0 = .worksheetCosWithJ0X87Product
    ∧ besseljCosinePublicationRoute .j1 = .worksheetCosPlainProduct := by
  native_decide

end OxFunc.Functions
