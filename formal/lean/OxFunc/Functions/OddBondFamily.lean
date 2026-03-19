import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def oddBondBaseMeta : FunctionMeta := {
  functionId := "FUNC.ODD_BOND_BASE"
  arity := Arity.exact 7
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

def oddfpriceMeta : FunctionMeta := {
  { oddBondBaseMeta with functionId := "FUNC.ODDFPRICE" } with
  arity := { min := 8, max := 9 }
}

def oddfyieldMeta : FunctionMeta := {
  { oddBondBaseMeta with functionId := "FUNC.ODDFYIELD" } with
  arity := { min := 8, max := 9 }
}

def oddlpriceMeta : FunctionMeta := {
  { oddBondBaseMeta with functionId := "FUNC.ODDLPRICE" } with
  arity := { min := 7, max := 8 }
}

def oddlyieldMeta : FunctionMeta := {
  { oddBondBaseMeta with functionId := "FUNC.ODDLYIELD" } with
  arity := { min := 7, max := 8 }
}

def oddBondSeed (fnName : String) : Option Rat :=
  if fnName = "ODDFPRICE" then some ((113597717474079 : Rat) / 1000000000000)
  else if fnName = "ODDFYIELD" then some ((1 : Rat) / 16)
  else if fnName = "ODDLPRICE" then some ((4993914300736067 : Rat) / 50000000000000)
  else if fnName = "ODDLYIELD" then some ((81 : Rat) / 2000)
  else none

theorem oddBond_seed_rows :
    oddBondSeed "ODDFPRICE" = some ((113597717474079 : Rat) / 1000000000000)
    ∧ oddBondSeed "ODDFYIELD" = some ((1 : Rat) / 16)
    ∧ oddBondSeed "ODDLPRICE" = some ((4993914300736067 : Rat) / 50000000000000)
    ∧ oddBondSeed "ODDLYIELD" = some ((81 : Rat) / 2000) := by
  native_decide

theorem oddBond_profiles :
    oddfpriceMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ oddfyieldMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ oddlpriceMeta.hostInteraction = HostInteractionClass.none
    ∧ oddlyieldMeta.threadSafety = ThreadSafetyClass.safePure := by
  simp [oddBondBaseMeta, oddfpriceMeta, oddfyieldMeta, oddlpriceMeta, oddlyieldMeta]

end OxFunc.Functions
