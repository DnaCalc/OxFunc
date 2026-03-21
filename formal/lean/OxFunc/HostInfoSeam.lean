namespace OxFunc

inductive CellInfoQuery where
  | address
  | row
  | col
  | contents
  | type
  | filename
  | format
  | color
  | parentheses
  | prefix
  | protect
  | width
  | isFormula
  deriving DecidableEq, Repr

inductive InfoQuery where
  | directory
  | numFile
  | origin
  | osVersion
  | recalc
  | release
  | system
  | memAvail
  | memUsed
  | totMem
  deriving DecidableEq, Repr

inductive SheetIdentitySpec where
  | currentSheet
  | reference (target : String)
  | sheetNameText (text : String)
  deriving DecidableEq, Repr

inductive SheetCountSpec where
  | workbook
  | reference (target : String)
  deriving DecidableEq, Repr

inductive WidthConversionFunction where
  | asc
  | dbcs
  | jis
  deriving DecidableEq, Repr

inductive WidthConversionMode where
  | passThrough
  | narrowBasicWidthAndKana
  | widenBasicWidthAndKana
  | unavailable
  deriving DecidableEq, Repr

structure TranslateRequest where
  text : String
  sourceLanguage : Option String
  targetLanguage : Option String
  deriving DecidableEq, Repr

inductive TranslateProviderResult where
  | text (value : String)
  | busy
  | capabilityDenied
  | providerError (code : String)
  deriving DecidableEq, Repr

structure AggregateCellContext where
  rowHiddenManual : Bool
  rowFilteredOut : Bool
  nestedSubtotalOrAggregate : Bool
  deriving DecidableEq, Repr

structure AggregateReferenceContext where
  rows : Nat
  cols : Nat
  cells : List AggregateCellContext
  deriving DecidableEq, Repr

def providerCanServeCellQuery (_q : CellInfoQuery) : Bool :=
  true

def providerCanServeFormulaTextQuery (_target : String) : Bool :=
  true

def providerCanServeSheetIdentity (_spec : SheetIdentitySpec) : Bool :=
  true

def providerCanServeSheetCount (_spec : SheetCountSpec) : Bool :=
  true

def providerCanServeAggregateReferenceContext (_target : String) : Bool :=
  true

def providerCanServeWidthConversionProfile (_fn : WidthConversionFunction) : Bool :=
  true

def providerCanServeTranslate (_request : TranslateRequest) : Bool :=
  true

def isExplicitReferenceHostQuery (q : CellInfoQuery) : Bool :=
  match q with
  | .filename | .format | .color | .parentheses | .prefix | .protect | .width | .isFormula =>
      true
  | .address | .row | .col | .contents | .type => false

theorem allCellInfoQueries_are_provider_addressable (q : CellInfoQuery) :
    providerCanServeCellQuery q = true := by
  cases q <;> rfl

theorem allFormulaTextQueries_are_provider_addressable (target : String) :
    providerCanServeFormulaTextQuery target = true := by
  rfl

theorem allSheetIdentityQueries_are_provider_addressable (spec : SheetIdentitySpec) :
    providerCanServeSheetIdentity spec = true := by
  cases spec <;> rfl

theorem allSheetCountQueries_are_provider_addressable (spec : SheetCountSpec) :
    providerCanServeSheetCount spec = true := by
  cases spec <;> rfl

theorem allAggregateReferenceQueries_are_provider_addressable (target : String) :
    providerCanServeAggregateReferenceContext target = true := by
  rfl

theorem allWidthConversionQueries_are_provider_addressable (fn : WidthConversionFunction) :
    providerCanServeWidthConversionProfile fn = true := by
  cases fn <;> rfl

theorem allTranslateQueries_are_provider_addressable (request : TranslateRequest) :
    providerCanServeTranslate request = true := by
  rfl

theorem explicitReferenceHostQueries_are_exactly_host_sensitive (q : CellInfoQuery) :
    isExplicitReferenceHostQuery q = true ↔
      q = .filename ∨ q = .format ∨ q = .color ∨ q = .parentheses
        ∨ q = .prefix ∨ q = .protect ∨ q = .width ∨ q = .isFormula := by
  cases q <;> simp [isExplicitReferenceHostQuery]

end OxFunc
