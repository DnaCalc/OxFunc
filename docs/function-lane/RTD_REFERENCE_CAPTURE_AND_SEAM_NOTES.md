# RTD Reference Capture And Seam Notes

## 1. Purpose
Record the local reference captures for `RTD` and the current OxFunc-side seam reading.

This note is intentionally narrow:
1. fetch and store the primary public references for `RTD`,
2. extract the parts that matter to OxFunc,
3. keep the OxFunc role minimal rather than collapsing host/application responsibilities into the function kernel.

## 2. Stored Reference Captures
Raw captures stored locally:
1. [ms-rtd-reference-aa140060-office10-20260320.html](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/ms-rtd-reference-aa140060-office10-20260320.html)
2. [excel-dna-rtd-tutorial-readme-20260320.md](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-tutorial-readme-20260320.md)
3. [excel-dna-rtd-tutorial-root-api-20260320.json](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-tutorial-root-api-20260320.json)
4. [excel-dna-rtd-functions-20260320.cs](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-functions-20260320.cs)
5. [excel-dna-rtd-server-20260320.cs](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-server-20260320.cs)

Primary source URLs:
1. `https://learn.microsoft.com/en-us/previous-versions/office/developer/office-xp/aa140060(v=office.10)`
2. `https://github.com/Excel-DNA/Tutorials/tree/master/SpecialTopics/RTD`

## 3. What The Microsoft RTD FAQ Pins
The archived Microsoft FAQ is still useful for the canonical RTD interaction model:
1. `RTD` is COM Automation based and uses an `IRtdServer` plus an `IRTDUpdateEvent` callback.
2. Excel tracks the cell-rooted `=RTD(...)` formulas and the server does not need to track worksheet locations itself.
3. Topic connection is established through `ConnectData`, updates are signalled through `UpdateNotify`, and Excel later pulls updated values with `RefreshData`.
4. Excel throttles update fetches and only refreshes when it is in a good state to change cell values.
5. The topic parameters are strings.
6. Saved workbook values interact with `ConnectData` through the `GetNewValues` flag.

## 4. What The Excel-DNA Tutorial Adds
The Excel-DNA tutorial adds useful implementation context:
1. wrapper functions commonly hide the raw `=RTD(...)` call behind user-facing functions,
2. the visible `RTD` function call effectively passes a COM ProgID plus a list of topic strings,
3. the RTD server can update topic values asynchronously and then notify Excel,
4. cleanup runs per topic and per server,
5. the callback/update path must be handled carefully on the Excel side.

The minimal sample in:
1. [excel-dna-rtd-functions-20260320.cs](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-functions-20260320.cs)
2. [excel-dna-rtd-server-20260320.cs](/C:/Work/DnaCalc/OxFunc/docs/function-lane/reference-captures/rtd/excel-dna-rtd-server-20260320.cs)

shows the clearest reference shape:
1. formula/wrapper emits `XlCall.RTD(progId, server, topic0, topic1, ...)`,
2. the server receives `IList<string> topicInfo`,
3. topic updates later arrive through `topic.UpdateValue(...)`.

## 5. OxFunc-Side Seam Reading
Current OxFunc reading:
1. OxFunc should own the semantic admission and shape of the `RTD` function call.
2. OxFunc should not own COM activation, topic registration tables, topic lifetime tracking, callback threading, update scheduling, or workbook/cell subscription maps.
3. Those responsibilities sit between OxFml and the higher-level host application.
4. OxFunc should only need enough seam surface to:
   - recognize `RTD`,
   - preserve the ProgID/server/topic-string payload honestly,
   - classify the function as external-provider and external-invalidation dependent,
   - accept a host-supplied current topic value when one is available,
   - distinguish capability-denied, provider-failed, disconnected, and normal-value states at the function result boundary.

## 6. Minimal Candidate OxFml <-> OxFunc Boundary
This is a candidate direction, not a locked interface:
1. prepared call surface:
   - `RtdRequest`
   - `prog_id: text`
   - `server_name: blank-preserved text`
   - `topic_strings: ordered text vector`
2. prepared evaluation context:
   - host supplies an `RtdProvider`-like resolution hook
   - any stable topic handle or subscription identity may exist above OxFunc, but OxFunc does not need to allocate or own it
3. value arrival surface:
   - OxFunc should see either a current resolved external value or a classified external-provider outcome
4. topic lifetime and update notification stay outside OxFunc

## 7. Immediate Modeling Consequence
For OxFunc, `RTD` is not primarily an algorithmic function.

It is better modeled as:
1. a function-call surface that creates or refers to an external topic subscription above OxFunc,
2. a host-managed invalidation/update channel,
3. and a value/result projection back into the formula from that host-managed state.

## 8. Open Questions For Dedicated RTD Work
These move to the dedicated `RTD` workset:
1. exact treatment of the second `server` argument on current Excel baselines,
2. classification of startup / no-server / disconnected / no-value-yet / provider-error outcomes,
3. whether workbook-saved prior values need explicit seam representation or remain a host-only concern,
4. whether OxFunc should ever see a stable topic handle, or only the resolved current value/result class.

## 9. Current Local Implementation
Current OxFunc-local first pass:
1. `crates/oxfunc_core/src/functions/rtd_fn.rs` implements `RtdRequest`, `RtdProvider`, and `RtdProviderResult`.
2. `RTD` is wired into the normal dispatch and export catalog.
3. The current result mapping is:
   - `Value(v)` -> `v`
   - `NoValueYet` -> `#N/A`
   - `CapabilityDenied` -> `#BLOCKED!`
   - `ConnectionFailed` -> `#CONNECT!`
   - `ProviderError(code)` -> `code`
4. This is intentionally only the OxFunc-local semantic boundary, not a full RTD host implementation.
