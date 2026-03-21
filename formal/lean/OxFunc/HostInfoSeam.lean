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

def providerCanServeCellQuery (_q : CellInfoQuery) : Bool :=
  true

def providerCanServeFormulaTextQuery (_target : String) : Bool :=
  true

def providerCanServeSheetIdentity (_spec : SheetIdentitySpec) : Bool :=
  true

def providerCanServeSheetCount (_spec : SheetCountSpec) : Bool :=
  true

def isExplicitReferenceHostQuery (q : CellInfoQuery) : Bool :=
  match q with
  | .filename | .format | .color | .parentheses | .prefix | .protect | .width => true
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

theorem explicitReferenceHostQueries_are_exactly_host_sensitive (q : CellInfoQuery) :
    isExplicitReferenceHostQuery q = true ↔
      q = .filename ∨ q = .format ∨ q = .color ∨ q = .parentheses
        ∨ q = .prefix ∨ q = .protect ∨ q = .width := by
  cases q <;> simp [isExplicitReferenceHostQuery]

end OxFunc
