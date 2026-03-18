import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def discreteDistBaseMeta : FunctionMeta := {
  functionId := "FUNC.DISCRETE_DIST_BASE"
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

def binomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.DIST"
  arity := Arity.exact 4
}

def binomDistRangeMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.DIST.RANGE"
  arity := { min := 3, max := 4 }
}

def binomInvMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOM.INV"
  arity := Arity.exact 3
}

def binomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.BINOMDIST"
  arity := Arity.exact 4
}

def critbinomMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.CRITBINOM"
  arity := Arity.exact 3
}

def poissonMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.POISSON"
  arity := Arity.exact 3
}

def poissonDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.POISSON.DIST"
  arity := Arity.exact 3
}

def hypgeomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.HYPGEOM.DIST"
  arity := Arity.exact 5
}

def hypgeomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.HYPGEOMDIST"
  arity := Arity.exact 4
}

def negbinomDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.NEGBINOM.DIST"
  arity := Arity.exact 4
}

def negbinomdistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.NEGBINOMDIST"
  arity := Arity.exact 3
}

def exponDistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.EXPON.DIST"
  arity := Arity.exact 3
}

def expondistMeta : FunctionMeta := {
  discreteDistBaseMeta with
  functionId := "FUNC.EXPONDIST"
  arity := Arity.exact 3
}

theorem discreteDistFamily_profiles :
    binomDistMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ binomDistRangeMeta.arity = { min := 3, max := 4 }
    ∧ binomInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ poissonMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ poissonDistMeta.arity = Arity.exact 3
    ∧ hypgeomDistMeta.arity = Arity.exact 5
    ∧ hypgeomdistMeta.arity = Arity.exact 4
    ∧ negbinomDistMeta.arity = Arity.exact 4
    ∧ negbinomdistMeta.arity = Arity.exact 3
    ∧ exponDistMeta.arity = Arity.exact 3
    ∧ expondistMeta.arity = Arity.exact 3 := by
  simp [
    discreteDistBaseMeta, binomDistMeta, binomDistRangeMeta, binomInvMeta, poissonMeta, poissonDistMeta, hypgeomDistMeta, hypgeomdistMeta,
    negbinomDistMeta, negbinomdistMeta, exponDistMeta, expondistMeta
  ]

end OxFunc.Functions

