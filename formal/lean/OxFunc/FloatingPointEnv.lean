namespace OxFunc

inductive FloatEnvironmentLayer where
  | rationalKernel
  | worksheetFormulaSurface
  | xllInteropSurface
  deriving DecidableEq, Repr

inductive WorksheetFloatOutcome where
  | visibleZero
  | visibleFinite
  | div0Error
  | numError
  | textShowsNegativeZero
  deriving DecidableEq, Repr

structure FloatObservationBinding where
  layer : FloatEnvironmentLayer
  outcome : WorksheetFloatOutcome
  evidenceId : String
  note : String
  deriving DecidableEq, Repr

end OxFunc
