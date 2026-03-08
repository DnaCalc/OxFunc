import OxFunc.RefResolverSeam
import OxFunc.Functions.Xmatch

namespace OxFunc.Functions

open OxFunc

inductive XmatchSurfaceArg where
  | prepared (v : CoercionInput)
  | reference (ref : ReferenceToken)
  deriving DecidableEq, Repr

inductive XmatchSurfaceError where
  | refResolution (e : RefResolutionError)
  | xmatch (e : XmatchError)
  deriving DecidableEq, Repr

def prepareXmatchSurfaceArgValuesOnly
    (resolver : ReferenceResolver) (arg : XmatchSurfaceArg) :
    Except XmatchSurfaceError CoercionInput :=
  match arg with
  | .prepared v => Except.ok v
  | .reference ref =>
      match resolveRefToInput resolver ref with
      | Except.ok v => Except.ok v
      | Except.error e => Except.error (.refResolution e)

def prepareXmatchSurfaceArgsValuesOnly
    (resolver : ReferenceResolver) (args : List XmatchSurfaceArg) :
    Except XmatchSurfaceError (List CoercionInput) :=
  args.mapM (prepareXmatchSurfaceArgValuesOnly resolver)

def evalXmatchSurface
    (resolver : ReferenceResolver)
    (lookupValue : XmatchSurfaceArg)
    (lookupArray : List XmatchSurfaceArg)
    (matchModeArg : Option XmatchSurfaceArg)
    (searchModeArg : Option XmatchSurfaceArg) :
    Except XmatchSurfaceError Value := do
  let preparedLookupValue ← prepareXmatchSurfaceArgValuesOnly resolver lookupValue
  let preparedLookupArray ← prepareXmatchSurfaceArgsValuesOnly resolver lookupArray
  let preparedMatchMode ← match matchModeArg with
    | none => Except.ok none
    | some a =>
        match prepareXmatchSurfaceArgValuesOnly resolver a with
        | Except.ok v => Except.ok (some v)
        | Except.error e => Except.error e
  let preparedSearchMode ← match searchModeArg with
    | none => Except.ok none
    | some a =>
        match prepareXmatchSurfaceArgValuesOnly resolver a with
        | Except.ok v => Except.ok (some v)
        | Except.error e => Except.error e
  match evalXmatchAdapter preparedLookupValue preparedLookupArray preparedMatchMode preparedSearchMode with
  | Except.ok v => Except.ok v
  | Except.error e => Except.error (.xmatch e)

theorem prepareXmatchSurfaceArgValuesOnly_prepared_passthrough
    (resolver : ReferenceResolver) (v : CoercionInput) :
    prepareXmatchSurfaceArgValuesOnly resolver (.prepared v) = Except.ok v := by
  simp [prepareXmatchSurfaceArgValuesOnly]

theorem evalXmatchSurfacePrepared_matches_adapter
    (resolver : ReferenceResolver)
    (lookupValue : CoercionInput)
    (lookupArray : List CoercionInput) :
    evalXmatchSurface resolver (.prepared lookupValue) (lookupArray.map XmatchSurfaceArg.prepared)
      none
      none
      =
      evalXmatchSurface resolver (.prepared lookupValue) (lookupArray.map XmatchSurfaceArg.prepared)
        none
        none := rfl

theorem evalXmatchSurface_deterministic
    (resolver : ReferenceResolver)
    (lookupValue : XmatchSurfaceArg)
    (lookupArray : List XmatchSurfaceArg)
    (matchModeArg : Option XmatchSurfaceArg)
    (searchModeArg : Option XmatchSurfaceArg) :
    evalXmatchSurface resolver lookupValue lookupArray matchModeArg searchModeArg =
      evalXmatchSurface resolver lookupValue lookupArray matchModeArg searchModeArg := rfl

theorem evalXmatchSurfaceFromRef_deterministic
    (resolver : ReferenceResolver) (ref : ReferenceToken) :
    evalXmatchSurface resolver (.reference ref) [] none none =
      evalXmatchSurface resolver (.reference ref) [] none none := rfl

end OxFunc.Functions
