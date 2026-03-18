import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def databaseMetaBase : FunctionMeta := {
  functionId := "FUNC.DATABASE_BASE"
  arity := Arity.exact 3
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def daverageMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DAVERAGE" }
def dcountMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DCOUNT" }
def dcountaMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DCOUNTA" }
def dgetMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DGET" }
def dmaxMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DMAX" }
def dminMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DMIN" }
def dproductMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DPRODUCT" }
def dstdevMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DSTDEV" }
def dstdevpMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DSTDEVP" }
def dsumMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DSUM" }
def dvarMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DVAR" }
def dvarpMeta : FunctionMeta := { databaseMetaBase with functionId := "FUNC.DVARP" }

theorem database_family_profiles :
    daverageMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ dcountMeta.arity = Arity.exact 3
    ∧ dgetMeta.hostInteraction = HostInteractionClass.none
    ∧ dstdevMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ dvarpMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [databaseMetaBase, daverageMeta, dcountMeta, dgetMeta, dstdevMeta, dvarpMeta]

end OxFunc.Functions
