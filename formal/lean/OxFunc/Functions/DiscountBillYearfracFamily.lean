import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def discountBillYearfracBaseMeta : FunctionMeta := {
  functionId := "FUNC.DISCOUNT_BILL_YEARFRAC_BASE"
  arity := Arity.exact 3
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

def discMeta : FunctionMeta := {
  { discountBillYearfracBaseMeta with functionId := "FUNC.DISC" } with
  arity := { min := 4, max := 5 }
}

def intrateMeta : FunctionMeta := {
  { discountBillYearfracBaseMeta with functionId := "FUNC.INTRATE" } with
  arity := { min := 4, max := 5 }
}

def receivedMeta : FunctionMeta := {
  { discountBillYearfracBaseMeta with functionId := "FUNC.RECEIVED" } with
  arity := { min := 4, max := 5 }
}

def pricediscMeta : FunctionMeta := {
  { discountBillYearfracBaseMeta with functionId := "FUNC.PRICEDISC" } with
  arity := { min := 4, max := 5 }
}

def tbilleqMeta : FunctionMeta := {
  discountBillYearfracBaseMeta with
  functionId := "FUNC.TBILLEQ"
}

def tbillpriceMeta : FunctionMeta := {
  discountBillYearfracBaseMeta with
  functionId := "FUNC.TBILLPRICE"
}

def tbillyieldMeta : FunctionMeta := {
  discountBillYearfracBaseMeta with
  functionId := "FUNC.TBILLYIELD"
}

def yearfracMeta : FunctionMeta := {
  { discountBillYearfracBaseMeta with functionId := "FUNC.YEARFRAC" } with
  arity := { min := 2, max := 3 }
}

def discountBillYearfracSeed (fnName : String) : Option Rat :=
  if fnName = "TBILLPRICE" then some ((1969 : Rat) / 20)
  else if fnName = "TBILLYIELD" then some ((457083064576659 : Rat) / 5000000000000000)
  else if fnName = "TBILLEQ" then some ((470756646216769 : Rat) / 5000000000000000)
  else if fnName = "YEARFRAC_ACTACT" then some ((57650273 : Rat) / 100000000)
  else none

theorem discountBillYearfrac_seed_rows :
    discountBillYearfracSeed "TBILLPRICE" = some ((1969 : Rat) / 20)
    ∧ discountBillYearfracSeed "TBILLYIELD" = some ((457083064576659 : Rat) / 5000000000000000)
    ∧ discountBillYearfracSeed "TBILLEQ" = some ((470756646216769 : Rat) / 5000000000000000)
    ∧ discountBillYearfracSeed "YEARFRAC_ACTACT" = some ((57650273 : Rat) / 100000000) := by
  native_decide

theorem discountBillYearfrac_profiles :
    discMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ intrateMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ receivedMeta.hostInteraction = HostInteractionClass.none
    ∧ pricediscMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ tbilleqMeta.surfaceFecDependencyProfile = FecDependencyProfile.none
    ∧ yearfracMeta.arity = { min := 2, max := 3 } := by
  simp [discountBillYearfracBaseMeta, discMeta, intrateMeta, receivedMeta, pricediscMeta,
    tbilleqMeta, yearfracMeta]

end OxFunc.Functions
