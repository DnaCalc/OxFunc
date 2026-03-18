import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def specialDistBaseMeta : FunctionMeta := {
  functionId := "FUNC.SPECIAL_DIST_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def erfMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.ERF"
  arity := { min := 1, max := 2 }
}

def erfPreciseMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.ERF.PRECISE"
}

def erfcMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.ERFC"
}

def erfcPreciseMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.ERFC.PRECISE"
}

def gammaMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.GAMMA"
}

def gammalnMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.GAMMALN"
}

def gammalnPreciseMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.GAMMALN.PRECISE"
}

def weibullMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.WEIBULL"
  arity := Arity.exact 4
}

def weibullDistMeta : FunctionMeta := {
  specialDistBaseMeta with
  functionId := "FUNC.WEIBULL.DIST"
  arity := Arity.exact 4
}

def erfSeed (lower : Rat) (upper : Option Rat) : Option Rat :=
  if lower = 1 ∧ upper = none then some ((421350396474857 : Rat) / 500000000000000)
  else if lower = 0 ∧ upper = some 1 then some ((421350396474857 : Rat) / 500000000000000)
  else if lower = 1 ∧ upper = some 2 then some ((76310736034619 : Rat) / 500000000000000)
  else none

def gammaSeed (x : Rat) : Option Rat :=
  if x = 5 then some 24
  else if x = (1 / 2 : Rat) then some ((886226925452758 : Rat) / 500000000000000)
  else none

def weibullSeed (x alpha beta : Rat) (cumulative : Bool) : Option Rat :=
  if x = 2 ∧ alpha = 3 ∧ beta = 4 ∧ cumulative = true then some ((117503097415405 : Rat) / 1000000000000000)
  else if x = 2 ∧ alpha = 3 ∧ beta = 4 ∧ cumulative = false then some ((165468169234612 : Rat) / 1000000000000000)
  else none

theorem specialDist_seed_rows :
    erfSeed 1 none = some ((421350396474857 : Rat) / 500000000000000)
    ∧ erfSeed 1 (some 2) = some ((76310736034619 : Rat) / 500000000000000)
    ∧ gammaSeed 5 = some 24
    ∧ gammaSeed (1 / 2 : Rat) = some ((886226925452758 : Rat) / 500000000000000)
    ∧ weibullSeed 2 3 4 true = some ((117503097415405 : Rat) / 1000000000000000) := by
  native_decide

theorem specialDistMeta_profiles :
    erfMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ erfPreciseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ erfcMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ erfcPreciseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ gammaMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ gammalnMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ gammalnPreciseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weibullMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weibullDistMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [
    specialDistBaseMeta, erfMeta, erfPreciseMeta, erfcMeta, erfcPreciseMeta, gammaMeta,
    gammalnMeta, gammalnPreciseMeta, weibullMeta, weibullDistMeta
  ]

end OxFunc.Functions
