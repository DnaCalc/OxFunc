import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cleanMeta : FunctionMeta := {
  functionId := "FUNC.CLEAN"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.none
  kernelSignatureClass := KernelSignatureClass.textToText
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def cleanRemovesCodepoint (n : Nat) : Bool :=
  n < 32 || n = 129 || n = 141 || n = 143 || n = 144 || n = 157

def cleanString (s : String) : String :=
  String.ofList <| s.toList.filter (fun c => !(cleanRemovesCodepoint c.val.toNat))

theorem cleanSeed_removes_seed_tab :
    cleanString "A\tB" = "AB" := by
  native_decide

theorem cleanSeed_removes_excel_c1_subset :
    cleanString (String.ofList [Char.ofNat 129, 'A', Char.ofNat 141, 'B']) = "AB" := by
  native_decide

theorem cleanSeed_preserves_char_127_and_zero_width_space :
    cleanString (String.ofList [Char.ofNat 127, Char.ofNat 8203, 'A']) =
      String.ofList [Char.ofNat 127, Char.ofNat 8203, 'A'] := by
  native_decide

theorem cleanMeta_profiles :
    cleanMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ cleanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [cleanMeta]

end OxFunc.Functions
