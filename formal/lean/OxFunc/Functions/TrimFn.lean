import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def trimMeta : FunctionMeta := {
  functionId := "FUNC.TRIM"
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

def trimLoop : Bool → Bool → List Char → List Char
  | _, _, [] => []
  | seen, pending, ' ' :: rest => trimLoop seen (seen || pending) rest
  | _, pending, ch :: rest =>
      (if pending then [' '] else []) ++ [ch] ++ trimLoop true false rest

def trimAsciiSpacesCore (chars : List Char) : List Char :=
  let body := chars.dropWhile (· = ' ')
  trimLoop false false body

theorem trimSeed_ascii_collapse :
    String.ofList (trimAsciiSpacesCore " A   B ".toList) = "A B" := by
  native_decide

theorem trimMeta_profiles :
    trimMeta.kernelSignatureClass = KernelSignatureClass.textToText
    ∧ trimMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [trimMeta]

end OxFunc.Functions
