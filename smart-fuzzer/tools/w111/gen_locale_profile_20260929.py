"""Typed discovery for the explicitly recorded en-ZA locale profile."""
import hashlib
import json
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE = "w111-locale-profile-20260929"
for fn, args in {
    "DATEVALUE":[g.t("2026/09/29")], "TIMEVALUE":[g.t("13:30")],
    "VALUE":[g.t("1 234.5")], "DOLLAR":[g.n(1234.5),g.n(2)],
    "FIXED":[g.n(1234.5),g.n(2),g.b(False)], "TEXT":[g.n(1234.5),g.t("0.00")],
    "ASC":[g.t("ABC１２３")], "DBCS":[g.t("ABC123")], "JIS":[g.t("ABC123")],
}.items():
    g.variants(fn,args)

for text in ["2026/09/29","2026-09-29","2026/9/2","1/2/2026","2026/2/1","29 Sep 2026",
             "September 29, 2026","26/9/29","49/1/1","50/1/1","2026/09/29 13:30",
             "1900/02/28","1900/02/29","1900/03/01","1899/12/31","9999/12/31",
             "2024/2/29","2025/2/29","2026/13/1","2026/1/0","0","1",""," "]:
    g.emit("DATEVALUE","date-text-"+str(len(g.cases)),[g.t(text)])
for text in ["0:00","00:00:00","13:30","23:59:59","24:00","25:00","12:60","12:59:60",
             "1:02:03.5","1 PM","1:30 PM","12 AM","12 PM","13:30 PM","2026/09/29 13:30",
             "0.5","1","-1:00",""," "]:
    g.emit("TIMEVALUE","time-text-"+str(len(g.cases)),[g.t(text)])
for text in ["1234.5","1 234.5","1\u00a0234.5","1\u202f234.5","1,234.5","1.234,5","R1 234.50",
             "R 1 234.50","-R1 234.50","(R1 234.50)","1 234.50R","12.5%","1e3","1e309",
             "2026/09/29","13:30","TRUE","FALSE","+2"," 2 ","0x10",""," "]:
    g.emit("VALUE","value-text-"+str(len(g.cases)),[g.t(text)])
for fn in ("DOLLAR","FIXED"):
    for value in [-1234.5,-2.675,-.005,0,.005,2.675,1234.5,1e20,2**53-1]:
        for decimals in [-3,-1,0,1,2,4]:
            g.emit(fn,"number-digits-"+str(len(g.cases)),[g.n(value),g.n(decimals)])
    for value in [g.missing(),g.blank(),g.t("2"),g.n(2.9),g.e()]:
        g.emit(fn,"digits-coercion-"+str(len(g.cases)),[g.n(1234.5),value])
for commas in [g.b(True),g.b(False),g.missing(),g.n(0),g.n(1),g.t("TRUE"),g.t(""),g.e()]:
    g.emit("FIXED","commas-"+str(len(g.cases)),[g.n(1234.5),g.n(2),commas])
for value in [0,1,-1,1234.567,45200.5625]:
    for code in ["General","0.00","#,##0.00","0%","0.00E+00","yyyy/mm/dd","hh:mm:ss",
                 '"R"#,##0.00','0.00;(0.00);"zero"','[>100]0.0;0.00',"@","0.000000000000000"]:
        g.emit("TEXT","format-"+str(len(g.cases)),[g.n(value),g.t(code)])
for fn in ("ASC","DBCS","JIS"):
    for text in ["ABC123","ＡＢＣ１２３","ｶﾀｶﾅ","カタカナ","\u3000","A B","éß","😀","あいう","ｶﾞ","ガ"]:
        g.emit(fn,"width-"+str(len(g.cases)),[g.t(text)])
for case in g.cases: case["case_id"] = case["case_id"].replace("w111typed-","w111locale-")
path = g.ROOT/"smart-fuzzer/cache/w111-locale-profile-cases-20260929.json"
doc = {"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0","tranche_id":g.TRANCHE,
       "generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
       "required_local_context":"--features oxfml-locale; --locale-profile en-ZA --locale-profile-record smart-fuzzer/cache/w111-locale-profile-20260929.json --use-recorded-locale-settings",
       "cases":g.cases,"tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c["case_id"] for c in g.cases]}]}
path.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
path.with_suffix(".jsonl").write_text("".join(json.dumps(case,ensure_ascii=False)+"\n" for case in g.cases),encoding="utf-8")
print(json.dumps({"path":str(path),"rows":len(g.cases),"functions":len({c['function_id'] for c in g.cases})}))
