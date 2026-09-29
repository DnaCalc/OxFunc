//! General kernel-expression research against retained public-interface captures.
//! Candidate coefficients use the production formatter, whose unresolved midpoint lane
//! remains independently recorded. No oracle-derived coefficient correction is used.
use oxfunc_core::functions::{cos::cos_kernel, sin::sin_kernel, surface_dispatch::eval_surface_value_call};
use oxfunc_core::functions::{cosh::cosh_kernel, sinh::sinh_kernel, tan::tan_kernel};
use oxfunc_core::functions::exp_fn::exp_kernel;
use oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER;
use oxfunc_core::value::{CalcValue, CoreValue};
use serde_json::{Value, json};
use std::collections::HashMap;

fn text(re: f64, im: f64) -> String {
    let result = eval_surface_value_call("FUNC.COMPLEX", &[CalcValue::number(re), CalcValue::number(im)],
        &NULL_REFERENCE_SYSTEM_PROVIDER, None, None, None, None).unwrap_or_else(CalcValue::error);
    match result.core() {
        CoreValue::Text(t) => format!("text:{}", t.to_string_lossy()),
        CoreValue::Error(code) => format!("error:{code:?}"),
        other => panic!("unexpected {other:?}"),
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("capture path");
    let capture: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    if capture.get("cases").is_some() {
        if std::env::args().any(|a|a=="--dispatch") {
            typed_dispatch(&capture,&std::env::args().nth(2).expect("typed excel.jsonl"));
            return;
        }
        typed_main(&capture, &std::env::args().nth(2).expect("typed excel.jsonl"));
        return;
    }
    let function = capture["function"].as_str().unwrap();
    if std::env::args().nth(2).as_deref()==Some("--dispatch") {
        let rows=capture["witnesses"].as_array().unwrap();
        let mut misses=Vec::new();
        for row in rows {
            let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|a|CalcValue::number(f64::from_bits(
                u64::from_str_radix(a.as_str().unwrap().strip_prefix("0x").unwrap(),16).unwrap()))).collect();
            let value=eval_surface_value_call(&format!("FUNC.{function}"),&args,&NULL_REFERENCE_SYSTEM_PROVIDER,None,None,None,None).unwrap_or_else(CalcValue::error);
            let actual=match value.core(){CoreValue::Text(t)=>format!("text:{}",t.to_string_lossy()),CoreValue::Error(e)=>format!("error:{e:?}"),other=>panic!("unsupported {other:?}")};
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"actual":actual,"expected":row["expected_bits"]}));}
        }
        println!("{}",json!({"function":function,"candidate":"current-dispatch","rows":rows.len(),"matches":rows.len()-misses.len(),"misses":misses}));
        return;
    }
    let modes: &[&str] = match function {
        "IMSQRT" => &["hypot-std-trig", "naive-std-trig", "hypot-excel-trig", "naive-excel-trig", "naive-ftz-excel-trig"],
        "IMDIV" => &["squared-denominator", "direct-division", "direct-division-ftz"],
        "IMCOS" | "IMSIN" => &["std-trig", "characterized-trig"],
        "IMCOSH" | "IMSINH" | "IMSECH" | "IMCSCH" => &["std-hyperbolic", "characterized-hyperbolic"],
        "IMTAN" | "IMCOT" => &["double-angle", "characterized-tangent"],
        _ => panic!("unsupported research function"),
    };
    for mode in modes {
        let mut misses = Vec::new();
        let rows = capture["witnesses"].as_array().unwrap();
        for row in rows {
            let args: Vec<f64> = row["args"].as_array().unwrap().iter().map(|arg| {
                f64::from_bits(u64::from_str_radix(arg.as_str().unwrap().strip_prefix("0x").unwrap(),16).unwrap())
            }).collect();
            let (re, im) = if function == "IMSQRT" {
                let square=args[0]*args[0];
                let magnitude = if mode.starts_with("hypot") {args[0].hypot(0.0)}
                    else if *mode=="naive-ftz-excel-trig" && square<f64::MIN_POSITIVE {0.0}
                    else {square.sqrt()};
                let radius = magnitude.sqrt();
                let angle = 0.0_f64.atan2(args[0]) / 2.0;
                if mode.ends_with("excel-trig") { (radius*cos_kernel(angle), radius*sin_kernel(angle)) }
                else { (radius*angle.cos(), radius*angle.sin()) }
            } else if ["IMCOSH","IMSINH","IMSECH","IMCSCH"].contains(&function) {
                let n=args[0];let sine=function=="IMSINH"||function=="IMCSCH";
                let base=if *mode=="std-hyperbolic" {if sine {n.sinh()} else {n.cosh()}}
                    else if sine {sinh_kernel(n)} else {cosh_kernel(n)};
                (if function=="IMCSCH"||function=="IMSECH" {1.0/base} else {base},0.0)
            } else if function=="IMTAN"||function=="IMCOT" {
                let n=args[0];
                let value=if *mode=="double-angle" {(2.0*n).sin()/(if function=="IMTAN" {1.0+(2.0*n).cos()} else {1.0-(2.0*n).cos()})}
                    else {let t=if n.abs()>=134217728.0 {f64::NAN} else {tan_kernel(n)};if function=="IMCOT" {1.0/t} else {t}};
                (value,0.0)
            } else if function=="IMCOS" || function=="IMSIN" {
                let n=args[0];
                let value=if *mode=="std-trig" {if function=="IMCOS" {n.cos()} else {n.sin()}}
                    else if n.abs() >= 134217728.0 {f64::NAN}
                    else if function=="IMCOS" {cos_kernel(n)} else {sin_kernel(n)};
                (value,0.0)
            } else if mode.starts_with("direct-division") {
                let value=args[0]/args[1];
                (if *mode=="direct-division-ftz" && value.abs()<f64::MIN_POSITIVE {0.0} else {value},0.0)
            }
            else { (args[0]*args[1]/(args[1]*args[1]), 0.0) };
            let actual = text(re, im);
            if actual != row["expected_bits"].as_str().unwrap() {
                misses.push(json!({"id":row["id"],"args":row["args"],"actual":actual,
                    "expected":row["expected_bits"],"coefficients":[format!("0x{:016x}",re.to_bits()),format!("0x{:016x}",im.to_bits())]}));
            }
        }
        println!("{}", json!({"function":function,"candidate":mode,"rows":rows.len(),"matches":rows.len()-misses.len(),"misses":misses}));
    }
}

fn parse_complex(raw: &str) -> (f64, f64) {
    if raw.ends_with(['i','j']) {
        let body=&raw[..raw.len()-1];
        let split=body.char_indices().rev().find(|(index,c)|
            *index>0 && (*c=='+'||*c=='-') && !matches!(body.as_bytes()[index-1],b'e'|b'E')).map(|(i,_)|i);
        let imaginary=|s:&str|match s {""|"+"=>1.0,"-"=>-1.0,_=>parse_number(s)};
        if let Some(i)=split {(parse_number(&body[..i]),imaginary(&body[i..]))}
        else {(0.0,imaginary(body))}
    } else {(parse_number(raw),0.0)}
}

fn parse_number(raw:&str)->f64 {
    if !std::env::args().any(|a|a=="--truncate15") {return raw.parse().unwrap();}
    let negative=raw.starts_with('-');let unsigned=raw.trim_start_matches(['+','-']);
    let (mantissa,exponent)=unsigned.split_once(['e','E']).map_or((unsigned,0),|(m,e)|(m,e.parse::<i32>().unwrap()));
    let decimal=mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let digits:String=mantissa.chars().filter(|c|*c!='.').collect();
    let first=digits.find(|c|c!='0').unwrap_or(digits.len());
    if first==digits.len(){return 0.0;}
    let significant=&digits[first..(first+15).min(digits.len())];
    let power=exponent+decimal-first as i32-significant.len() as i32;
    format!("{}{}e{}",if negative {"-"} else {""},significant,power).parse().unwrap()
}

fn typed_main(capture: &Value, answer_path: &str) {
    let answers: HashMap<String,Value>=std::fs::read_to_string(answer_path).unwrap().lines()
        .map(|line|serde_json::from_str::<Value>(line).unwrap())
        .map(|row|(row["case_id"].as_str().unwrap().to_owned(),row["outcome"].clone())).collect();
    for function in ["IMSQRT","IMDIV","IMCOS","IMSIN","IMTAN","IMCOT","IMARGUMENT"] {
        let modes: &[&str]=match function {
            "IMSQRT"=>&["hypot-std-trig","naive-excel-trig","naive-ftz-excel-trig","term-ftz-sum-overflow-zero-excel-trig","term-ftz-sum-overflow-zero-ratio-angle","term-ftz-sum-overflow-zero-ratio-overflow-angle","term-ftz-sum-overflow-zero-ratio-overflow-angle-extended-products","term-ftz-sum-overflow-zero-ratio-overflow-angle-extended-ratio","term-ftz-sum-overflow-zero-ratio-overflow-angle-extended-sqrt","term-ftz-sum-overflow-zero-ratio-overflow-angle-fpatan"],
            "IMDIV"=>&["squared-denominator","smith-division","real-denominator-direct"],
            "IMARGUMENT"=>&["std-ratio-angle","fpatan-ratio-angle","extended-ratio-fpatan"],
            "IMCOS"|"IMSIN"=>&["std-products","characterized-circular-products","characterized-all-products","characterized-exp-pair","characterized-exp-reciprocal","characterized-exp-backend-pair","characterized-exp-backend-reciprocal","characterized-exp-backend-extended-reciprocal"],
            _=>&["std-double-angle","characterized-double-angle","characterized-exponential-double-angle","characterized-tangent-identity","characterized-sincos-division","characterized-squared-identity","characterized-squared-hyperbolic-identity","characterized-squared-extended-identity","characterized-tanh-identity","characterized-tangent-sinh-identity"],
        };
        for mode in modes {
            let mut misses=Vec::new();let mut count=0;
            for row in capture["cases"].as_array().unwrap().iter().filter(|r|r["canonical_surface_name"]==function) {
                count+=1;
                let args: Vec<_>=row["args"].as_array().unwrap().iter().map(|a|parse_complex(a["value"].as_str().unwrap())).collect();
                let (a,b)=args[0];
                let (re,im)=if function=="IMSQRT" {
                    let square=if mode.starts_with("term-ftz-sum-overflow-zero") {
                        let flush=|x:f64|if x<f64::MIN_POSITIVE {0.0} else {x};
                        let p=flush(if mode.ends_with("extended-products") {extended_product(a,a)} else {a*a});
                        let q=flush(if mode.ends_with("extended-products") {extended_product(b,b)} else {b*b});let sum=p+q;
                        if p.is_finite()&&q.is_finite()&&!sum.is_finite() {0.0} else {sum}
                    } else {a*a+b*b};
                    let radius=if *mode=="hypot-std-trig" {a.hypot(b).sqrt()}
                        else if *mode=="naive-ftz-excel-trig" && square<f64::MIN_POSITIVE {0.0}
                        else if mode.ends_with("extended-sqrt") {extended_sqrt(extended_sqrt(square))}
                        else {square.sqrt().sqrt()};
                    let angle=if mode.starts_with("term-ftz-sum-overflow-zero-ratio-") {
                        let b=if b==0.0 {0.0} else {b};
                        (if a==0.0 {if b==0.0 {0.0} else {b.signum()*std::f64::consts::FRAC_PI_2}}
                        else if mode.contains("ratio-overflow-angle") && !(b/a).is_finite() {0.0}
                        else {let ratio=if mode.ends_with("extended-ratio") {extended_quotient(b,a)} else {b/a};
                            let base=if mode.ends_with("fpatan") {fpatan(ratio)} else {ratio.atan()}; if a<0.0 {if b<0.0 {base-std::f64::consts::PI} else {base+std::f64::consts::PI}} else {base}})/2.0
                    } else if *mode=="term-ftz-sum-overflow-zero-excel-trig" {
                        (if b==0.0 {0.0} else {b}).atan2(if a==0.0 {0.0} else {a})/2.0
                    } else {b.atan2(a)/2.0};
                    if *mode=="hypot-std-trig" {(radius*angle.cos(),radius*angle.sin())}
                    else if mode.ends_with("extended-products") {(extended_product(radius,cos_kernel(angle)),extended_product(radius,sin_kernel(angle)))}
                    else {(radius*cos_kernel(angle),radius*sin_kernel(angle))}
                } else if function=="IMARGUMENT" {
                    let b=if b==0.0 {0.0} else {b};
                    let angle=if a==0.0 {if b==0.0 {0.0} else {b.signum()*std::f64::consts::FRAC_PI_2}}
                        else {let r=if mode.starts_with("extended-ratio") {extended_quotient(b,a)} else {b/a};
                            let base=if *mode=="std-ratio-angle" {r.atan()} else {fpatan(r)};
                            if a<0.0 {if b<0.0 {base-std::f64::consts::PI} else {base+std::f64::consts::PI}} else {base}};
                    (if angle.abs()<f64::MIN_POSITIVE {0.0} else {angle},0.0)
                } else if function=="IMCOS"||function=="IMSIN" {
                    let (s,c)=if *mode=="std-products" {(a.sin(),a.cos())}
                        else if a.abs()>=134217728.0 {(f64::NAN,f64::NAN)}
                        else {(sin_kernel(a),cos_kernel(a))};
                    let (sh,ch)=if mode.starts_with("characterized-exp-") {
                        let backend=|x:f64|if mode.contains("backend") {exp_kernel(x)} else {x.exp()};
                        let p=backend(b);let q=if mode.ends_with("pair") {backend(-b)}
                            else if mode.contains("extended-reciprocal") {extended_reciprocal(p)} else {1.0/p};
                        ((p-q)/2.0,(p+q)/2.0)
                    } else if *mode=="characterized-all-products" {(sinh_kernel(b),cosh_kernel(b))} else {(b.sinh(),b.cosh())};
                    if function=="IMSIN" {(s*ch,c*sh)} else {(c*ch,-s*sh)}
                } else if function=="IMTAN"||function=="IMCOT" {
                    let pair=if mode.ends_with("double-angle") {
                        let (s,c)=if *mode=="std-double-angle" {((2.0*a).sin(),(2.0*a).cos())}
                            else if (if mode.contains("exponential") {a} else {2.0*a}).abs()>=134217728.0 {(f64::NAN,f64::NAN)}
                            else {(sin_kernel(2.0*a),cos_kernel(2.0*a))};
                        let (sh,ch)=if mode.contains("exponential") {let p=exp_kernel(2.0*b);let q=extended_reciprocal(p);((p-q)/2.0,(p+q)/2.0)} else {((2.0*b).sinh(),(2.0*b).cosh())};
                        let denom=ch+if function=="IMTAN" {c} else {-c};
                        (s/denom,if function=="IMTAN" {sh/denom} else {-sh/denom})
                    } else if mode.starts_with("characterized-squared-") {
                        let (s,c)=if a.abs()>=134217728.0 {(f64::NAN,f64::NAN)} else {(sin_kernel(a),cos_kernel(a))};
                        let (sh,ch)=if *mode=="characterized-squared-identity" {(b.sinh(),b.cosh())} else {(sinh_kernel(b),cosh_kernel(b))};
                        if mode.contains("extended") {extended_squared_identity(s,c,sh,ch)}
                        else {let den=c*c+sh*sh;(s*c/den,sh*ch/den)}
                    } else if *mode=="characterized-tangent-sinh-identity" {
                        let t=if a.abs()>=134217728.0 {f64::NAN} else {tan_kernel(a)};
                        let sh=sinh_kernel(b);let ch=cosh_kernel(b);let den=ch*ch+t*t*sh*sh;
                        (t/den,(1.0+t*t)*sh*ch/den)
                    } else if *mode=="characterized-tanh-identity" {
                        let t=if a.abs()>=134217728.0 {f64::NAN} else {tan_kernel(a)};
                        let u=b.tanh();let den=1.0+t*t*u*u;
                        (t*(1.0-u*u)/den,u*(1.0+t*t)/den)
                    } else if *mode=="characterized-tangent-identity" {
                        let t=if a.abs()>=134217728.0 {f64::NAN} else {tan_kernel(a)};
                        let u=b.tanh(); smith(t,u,1.0,-t*u)
                    } else {
                        let (s,c)=if a.abs()>=134217728.0 {(f64::NAN,f64::NAN)} else {(sin_kernel(a),cos_kernel(a))};
                        smith(s*b.cosh(),c*b.sinh(),c*b.cosh(),-s*b.sinh())
                    };
                    if function=="IMCOT" && !mode.ends_with("double-angle") {smith(1.0,0.0,pair.0,pair.1)} else {pair}
                } else {
                    let (c,d)=args[1];
                    if *mode=="smith-division" {
                        if c.abs()>=d.abs() {
                            let ratio=d/c;let denominator=c+d*ratio;
                            ((a+b*ratio)/denominator,(b-a*ratio)/denominator)
                        } else {
                            let ratio=c/d;let denominator=d+c*ratio;
                            ((a*ratio+b)/denominator,(b*ratio-a)/denominator)
                        }
                    } else if *mode=="real-denominator-direct" && d==0.0 {(a/c,b/c)}
                    else {let denominator=c*c+d*d;((a*c+b*d)/denominator,(b*c-a*d)/denominator)}
                };
                let mut actual=if function=="IMARGUMENT" {
                    if (a==0.0&&b==0.0)||(a!=0.0&&(b/a).is_infinite()) {"error:Div0".to_owned()}
                    else {format!("number:0x{:016x}",re.to_bits())}
                } else {text(re,im)};
                // Inputs in this corpus have a common explicit suffix or none.
                if actual.starts_with("text:") && row["args"].as_array().unwrap().iter().any(|a|a["value"].as_str().unwrap().ends_with('j')) {
                    actual=actual.replace('i',"j");
                }
                let outcome=&answers[row["case_id"].as_str().unwrap()];
                let expected=match outcome["kind"].as_str().unwrap() {
                    "number"=>format!("number:{}",outcome["bits_hex"].as_str().unwrap()),
                    "text"=>format!("text:{}",outcome["value"].as_str().unwrap()),
                    "error"=>format!("error:{}",outcome["code"].as_str().unwrap()),
                    _=>panic!("unsupported outcome {outcome}"),
                };
                if actual!=expected {misses.push(json!({"id":row["case_id"],"args":row["args"],"expected":expected,"actual":actual,
                    "coefficients":[format!("0x{:016x}",re.to_bits()),format!("0x{:016x}",im.to_bits())]}));}
            }
            println!("{}",json!({"function":function,"candidate":mode,"input_policy":if std::env::args().any(|a|a=="--truncate15") {"truncate15-hypothesis"} else {"binary64-exact-parse"},"rows":count,"matches":count-misses.len(),"misses":misses}));
        }
    }
}

fn smith(a:f64,b:f64,c:f64,d:f64)->(f64,f64) {
    if c.abs()>=d.abs() {let r=d/c;let den=c+d*r;((a+b*r)/den,(b-a*r)/den)}
    else {let r=c/d;let den=d+c*r;((a*r+b)/den,(b*r-a)/den)}
}

fn extended_reciprocal(value:f64)->f64 {
    use oxfunc_core::excel_numeric::research::{ext_one,ext_from_f64,ext_to_f64,ext_div,CW_PC64_RN};
    ext_to_f64(&ext_div(&ext_one(),&ext_from_f64(value),CW_PC64_RN),CW_PC64_RN)
}

fn extended_product(a:f64,b:f64)->f64 {
    use oxfunc_core::excel_numeric::research::{ext_from_f64,ext_to_f64,ext_mul,CW_PC64_RN};
    ext_to_f64(&ext_mul(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN)
}
fn extended_quotient(a:f64,b:f64)->f64 {
    use oxfunc_core::excel_numeric::research::{ext_from_f64,ext_to_f64,ext_div,CW_PC64_RN};
    ext_to_f64(&ext_div(&ext_from_f64(a),&ext_from_f64(b),CW_PC64_RN),CW_PC64_RN)
}
fn extended_sqrt(a:f64)->f64 {
    use oxfunc_core::excel_numeric::research::{ext_from_f64,ext_to_f64,ext_sqrt,CW_PC64_RN};
    ext_to_f64(&ext_sqrt(&ext_from_f64(a),CW_PC64_RN),CW_PC64_RN)
}

// Public ISA operation used only to test an arithmetic graph against captured values.
// It saves/restores the floating-point control word and balances the x87 stack.
fn fpatan(value:f64)->f64 {
    let mut old=0_u16;let control=0x037f_u16;let mut out=0.0_f64;
    unsafe {std::arch::asm!(
        "fnstcw word ptr [{old}]", "fldcw word ptr [{control}]",
        "fld qword ptr [{value}]", "fld1", "fpatan", "fstp qword ptr [{out}]",
        "fldcw word ptr [{old}]",
        old=in(reg)&mut old,control=in(reg)&control,value=in(reg)&value,out=in(reg)&mut out,
        options(nostack,preserves_flags));}
    out
}

fn extended_squared_identity(s:f64,c:f64,sh:f64,ch:f64)->(f64,f64) {
    use oxfunc_core::excel_numeric::research::{ext_from_f64 as from,ext_to_f64 as to,ext_mul as mul,ext_add as add,ext_div as div,CW_PC64_RN as cw};
    let den=add(&mul(&from(c),&from(c),cw),&mul(&from(sh),&from(sh),cw),cw);
    (to(&div(&mul(&from(s),&from(c),cw),&den,cw),cw),to(&div(&mul(&from(sh),&from(ch),cw),&den,cw),cw))
}

fn typed_dispatch(capture:&Value,answer_path:&str) {
    let answers:HashMap<String,Value>=std::fs::read_to_string(answer_path).unwrap().lines()
        .map(|line|serde_json::from_str::<Value>(line).unwrap())
        .map(|r|(r["case_id"].as_str().unwrap().to_owned(),r)).collect();
    let mut functions=std::collections::BTreeMap::<String,(usize,Vec<Value>)>::new();
    for row in capture["cases"].as_array().unwrap() {
        let function=row["canonical_surface_name"].as_str().unwrap();
        let oracle=&answers[row["case_id"].as_str().unwrap()];
        assert_eq!(oracle["formula_text"],row["formula_text"]);
        assert_eq!(oracle["execution_status"],"ok");
        let args:Vec<_>=row["args"].as_array().unwrap().iter().map(|a|{
            assert_eq!(a["kind"],"text");
            CalcValue::text(oxfunc_core::value::ExcelText::from_utf16_code_units(a["value"].as_str().unwrap().encode_utf16().collect()))
        }).collect();
        let result=eval_surface_value_call(&format!("FUNC.{function}"),&args,&NULL_REFERENCE_SYSTEM_PROVIDER,None,None,None,None)
            .unwrap_or_else(CalcValue::error);
        let actual=match result.core(){
            CoreValue::Number(n)=>format!("number:0x{:016x}",n.to_bits()),
            CoreValue::Text(t)=>format!("text:{}",t.to_string_lossy()),
            CoreValue::Error(e)=>format!("error:{e:?}"),other=>panic!("unsupported {other:?}"),
        };
        let entry=functions.entry(function.to_owned()).or_default();entry.0+=1;
        if actual!=oracle["outcome"]["digest_payload"].as_str().unwrap() {
            entry.1.push(json!({"id":row["case_id"],"args":row["args"],"actual":actual,"expected":oracle["outcome"]["digest_payload"]}));
        }
    }
    for (function,(rows,misses)) in functions {println!("{}",json!({"function":function,"candidate":"current-typed-dispatch","rows":rows,"matches":rows-misses.len(),"misses":misses}));}
}
