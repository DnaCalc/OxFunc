import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def opUnaryNumericBaseMeta : FunctionMeta := {
  functionId := "FUNC.OP_UNARY_NUMERIC_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.numToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def opBinaryNumericBaseMeta : FunctionMeta := {
  functionId := "FUNC.OP_BINARY_NUMERIC_BASE"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def opUnaryPlusMeta : FunctionMeta := { opUnaryNumericBaseMeta with functionId := "FUNC.OP_UNARY_PLUS" }
def opNegateMeta : FunctionMeta := { opUnaryNumericBaseMeta with functionId := "FUNC.OP_NEGATE" }
def opPercentMeta : FunctionMeta := { opUnaryNumericBaseMeta with functionId := "FUNC.OP_PERCENT" }

def opSubtractMeta : FunctionMeta := { opBinaryNumericBaseMeta with functionId := "FUNC.OP_SUBTRACT" }
def opMultiplyMeta : FunctionMeta := { opBinaryNumericBaseMeta with functionId := "FUNC.OP_MULTIPLY" }
def opDivideMeta : FunctionMeta := { opBinaryNumericBaseMeta with functionId := "FUNC.OP_DIVIDE" }
def opPowerMeta : FunctionMeta := { opBinaryNumericBaseMeta with functionId := "FUNC.OP_POWER" }

def opUnaryPlusKernel (x : Rat) : Rat := x
def opNegateKernel (x : Rat) : Rat := -x
def opPercentKernel (x : Rat) : Rat := x / 100
def opSubtractKernel (lhs rhs : Rat) : Rat := lhs - rhs
def opMultiplyKernel (lhs rhs : Rat) : Rat := lhs * rhs

theorem operatorArithmeticFamily_meta_profiles :
    opUnaryPlusMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ opNegateMeta.coercionLiftProfile =
        CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ opPercentMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ opSubtractMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ opMultiplyMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ opDivideMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ opPowerMeta.hostInteraction = HostInteractionClass.none := by
  simp [
    opUnaryNumericBaseMeta,
    opBinaryNumericBaseMeta,
    opUnaryPlusMeta,
    opNegateMeta,
    opPercentMeta,
    opSubtractMeta,
    opMultiplyMeta,
    opDivideMeta,
    opPowerMeta
  ]

theorem operatorArithmeticFamily_ids :
    opUnaryPlusMeta.functionId = "FUNC.OP_UNARY_PLUS"
    ∧ opNegateMeta.functionId = "FUNC.OP_NEGATE"
    ∧ opPercentMeta.functionId = "FUNC.OP_PERCENT"
    ∧ opSubtractMeta.functionId = "FUNC.OP_SUBTRACT"
    ∧ opMultiplyMeta.functionId = "FUNC.OP_MULTIPLY"
    ∧ opDivideMeta.functionId = "FUNC.OP_DIVIDE"
    ∧ opPowerMeta.functionId = "FUNC.OP_POWER" := by
  simp [
    opUnaryPlusMeta,
    opNegateMeta,
    opPercentMeta,
    opSubtractMeta,
    opMultiplyMeta,
    opDivideMeta,
    opPowerMeta
  ]

theorem opUnaryPlusKernel_identity (x : Rat) :
    opUnaryPlusKernel x = x := rfl

theorem opNegateKernel_negates (x : Rat) :
    opNegateKernel x = -x := rfl

theorem opPercentKernel_scales (x : Rat) :
    opPercentKernel x = x / 100 := rfl

theorem opSubtractKernel_subtracts (lhs rhs : Rat) :
    opSubtractKernel lhs rhs = lhs - rhs := rfl

theorem opMultiplyKernel_multiplies (lhs rhs : Rat) :
    opMultiplyKernel lhs rhs = lhs * rhs := rfl

end OxFunc.Functions
