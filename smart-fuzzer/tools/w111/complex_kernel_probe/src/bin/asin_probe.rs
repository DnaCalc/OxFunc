//! Standard inverse-trigonometric identities with explicit arithmetic staging.
use oxfunc_core::excel_numeric::research::{Ext80,ext_from_f64 as from,ext_to_f64 as to,
    ext_add,ext_sub,ext_mul,ext_div,ext_sqrt,ext_pi,CW_PC64_RN as CW,CW_PC53_RN};
use serde_json::{Value,json};

fn op(a:&Ext80,b:&Ext80,operation:char,stage:&str)->Ext80 {
    if stage=="binary64" {
        let (a,b)=(to(a,CW),to(b,CW));
        return from(match operation {'+'=>a+b,'-'=>a-b,'*'=>a*b,'/'=>a/b,_=>unreachable!()});
    }
    let result=match operation {'+'=>ext_add(a,b,CW),'-'=>ext_sub(a,b,CW),'*'=>ext_mul(a,b,CW),'/'=>ext_div(a,b,CW),_=>unreachable!()};
    if stage=="stored-rn64" {from(to(&result,CW))} else {result}
}
fn sqrt(a:&Ext80,stage:&str)->Ext80 {
    if stage=="binary64" {return from(to(a,CW).sqrt());}
    let result=ext_sqrt(a,CW);
    if stage=="stored-rn64" {from(to(&result,CW))} else {result}
}
fn fpatan(y:&Ext80,x:&Ext80)->f64 {
    let mut old=0_u16;let mut out=0.0;
    unsafe {std::arch::asm!(
        "fnstcw word ptr [{old}]","fldcw word ptr [{control}]",
        "fld tbyte ptr [{y}]","fld tbyte ptr [{x}]","fpatan","fstp qword ptr [{out}]",
        "fldcw word ptr [{old}]",old=in(reg)&mut old,control=in(reg)&CW,
        y=in(reg)y.0.as_ptr(),x=in(reg)x.0.as_ptr(),out=in(reg)&mut out,
        options(nostack,preserves_flags));}
    out
}
fn fpatan_ext(y:&Ext80,x:&Ext80)->Ext80 {
    let mut old=0_u16;let mut out=Ext80([0;10]);
    unsafe {std::arch::asm!(
        "fnstcw word ptr [{old}]","fldcw word ptr [{control}]",
        "fld tbyte ptr [{y}]","fld tbyte ptr [{x}]","fpatan","fstp tbyte ptr [{out}]",
        "fldcw word ptr [{old}]",old=in(reg)&mut old,control=in(reg)&CW,
        y=in(reg)y.0.as_ptr(),x=in(reg)x.0.as_ptr(),out=in(reg)out.0.as_mut_ptr(),
        options(nostack,preserves_flags));}
    out
}
fn fpatan_reduced(n:f64)->f64 {
    if n.abs()<=1.0 {return fpatan(&from(n),&from(1.0));}
    let angle=fpatan_ext(&from(1.0),&from(n.abs()));
    to(&ext_sub(&ext_div(&ext_pi(),&from(2.0),CW),&angle,CW),CW).copysign(n)
}
fn complementary(n:f64,form:&str,stage:&str,pi_kind:&str,store_angle:bool)->f64 {
    let one=from(1.0);let two=from(2.0);let x=from(n);
    let pi=if pi_kind=="extended-pi" {ext_pi()} else {from(std::f64::consts::PI)};
    let half_pi=ext_div(&pi,&two,CW);
    let theta=if form=="half-angle-complement" {
        let ratio=op(&op(&one,&x,'-',stage),&op(&one,&x,'+',stage),'/',stage);
        ext_mul(&fpatan_ext(&sqrt(&ratio,stage),&one),&two,CW)
    } else if form=="inverse-half-angle-complement" {
        let ratio=op(&op(&one,&x,'+',stage),&op(&one,&x,'-',stage),'/',stage);
        ext_mul(&fpatan_ext(&sqrt(&ratio,stage),&one),&two,CW)
    } else {
        let product=if form=="difference-complement" {
            op(&one,&op(&x,&x,'*',stage),'-',stage)
        } else {op(&op(&one,&x,'-',stage),&op(&one,&x,'+',stage),'*',stage)};
        fpatan_ext(&sqrt(&product,stage),&x)
    };
    let theta=if store_angle {from(to(&theta,CW))} else {theta};
    if form=="inverse-half-angle-complement" {to(&ext_sub(&theta,&half_pi,CW),CW)}
    else {to(&ext_sub(&half_pi,&theta,CW),CW)}
}
fn predict(n:f64,form:&str,stage:&str,angle:&str,backend:&str)->f64 {
    if form=="platform-asin" {return n.asin();}
    let x=from(n);let one=from(1.0);
    let squared=if form=="product" {op(&op(&one,&x,'-',stage),&op(&one,&x,'+',stage),'*',stage)}
        else if form=="expanded-left" {let y=op(&one,&x,'-',stage);op(&y,&op(&x,&y,'*',stage),'+',stage)}
        else if form=="expanded-right" {let y=op(&one,&x,'+',stage);op(&y,&op(&x,&y,'*',stage),'-',stage)}
        else if form=="reused-minus" {let y=op(&one,&x,'-',stage);op(&y,&op(&from(2.0),&y,'-',stage),'*',stage)}
        else if form=="reused-plus" {let y=op(&one,&x,'+',stage);op(&y,&op(&from(2.0),&y,'-',stage),'*',stage)}
        else {op(&one,&op(&x,&x,'*',stage),'-',stage)};
    let mut denominator=sqrt(&squared,stage);
    if form=="half-angle" {denominator=op(&one,&denominator,'+',stage);}
    let (y,x)=if angle=="ratio" {(op(&x,&denominator,'/',stage),one)}
        else if angle=="reciprocal-product" {(op(&x,&op(&one,&denominator,'/',stage),'*',stage),one)}
        else {(x,denominator)};
    let angle=if backend=="fpatan" {fpatan(&y,&x)}
        else if backend=="fpatan-reduced" {fpatan_reduced(to(&y,CW)/to(&x,CW))}
        else if backend=="platform-atan" {(to(&y,CW)/to(&x,CW)).atan()}
        else {to(&y,CW).atan2(to(&x,CW))};
    if form=="half-angle" {angle*2.0} else {angle}
}
fn barrier_graph(n:f64,mask:u8,direct:bool)->f64 {
    let rounded=|x:Ext80,step:u8|if mask&(1<<step)!=0 {from(to(&x,CW))} else {x};
    let x=from(n);let one=from(1.0);
    let left=rounded(ext_sub(&one,&x,CW),0);
    let right=rounded(ext_add(&one,&x,CW),1);
    let product=rounded(ext_mul(&left,&right,CW),2);
    let root=rounded(ext_sqrt(&product,CW),3);
    if direct {return fpatan(&x,&root);}
    let ratio=rounded(ext_div(&x,&root,CW),4);
    fpatan(&ratio,&one)
}
fn split_root_graph(n:f64,mask:u8)->f64 {
    let rounded=|x:Ext80,step:u8|if mask&(1<<step)!=0 {from(to(&x,CW))} else {x};
    let x=from(n);let one=from(1.0);
    let left=rounded(ext_sub(&one,&x,CW),0);
    let right=rounded(ext_add(&one,&x,CW),1);
    let left=rounded(ext_sqrt(&left,CW),2);
    let right=rounded(ext_sqrt(&right,CW),3);
    let product=rounded(ext_mul(&left,&right,CW),4);
    let ratio=rounded(ext_div(&x,&product,CW),5);
    fpatan(&ratio,&one)
}
fn split_division_graph(n:f64,mask:u8,reverse:bool)->f64 {
    let rounded=|x:Ext80,step:u8|if mask&(1<<step)!=0 {from(to(&x,CW))} else {x};
    let x=from(n);let one=from(1.0);
    let left=rounded(ext_sub(&one,&x,CW),0);
    let right=rounded(ext_add(&one,&x,CW),1);
    let left=rounded(ext_sqrt(&left,CW),2);
    let right=rounded(ext_sqrt(&right,CW),3);
    let (first,second)=if reverse {(right,left)} else {(left,right)};
    let quotient=rounded(ext_div(&x,&first,CW),4);
    let ratio=rounded(ext_div(&quotient,&second,CW),5);
    fpatan(&ratio,&one)
}
fn precision_graph(n:f64,mut mode:usize)->f64 {
    let mut modes=[0;5];for item in &mut modes {*item=mode%3;mode/=3;}
    let control=|step:usize|if modes[step]==2 {CW_PC53_RN} else {CW};
    let rounded=|x:Ext80,step:usize|if modes[step]==1 {from(to(&x,CW))} else {x};
    let x=from(n);let one=from(1.0);
    let left=rounded(ext_sub(&one,&x,control(0)),0);
    let right=rounded(ext_add(&one,&x,control(1)),1);
    let product=rounded(ext_mul(&left,&right,control(2)),2);
    let root=rounded(ext_sqrt(&product,control(3)),3);
    let ratio=rounded(ext_div(&x,&root,control(4)),4);
    fpatan(&ratio,&one)
}
fn reciprocal_angle_graph(n:f64,stage:&str,pi_kind:&str,stored:bool)->f64 {
    if n==0.0 {return n;}
    let x=from(n);let one=from(1.0);
    let product=op(&op(&one,&x,'-',stage),&op(&one,&x,'+',stage),'*',stage);
    let inverse=op(&sqrt(&product,stage),&x,'/',stage);
    let theta=fpatan_ext(&one,&inverse);
    let theta=if stored {from(to(&theta,CW))} else {theta};
    let pi=if pi_kind=="extended-pi" {ext_pi()} else {from(std::f64::consts::PI)};
    if n<0.0 {to(&ext_sub(&theta,&pi,CW),CW)} else {to(&theta,CW)}
}
fn main() {
    let path=std::env::args().nth(1).unwrap();
    let capture:Value=serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    if std::env::args().any(|a|a=="--find-angle-differences") {
        let mut centers=Vec::new();
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let direct=predict(x,"reused-minus","stored-rn64","ratio","fpatan");
            let reduced=predict(x,"reused-minus","stored-rn64","ratio","fpatan-reduced");
            if direct.to_bits()!=reduced.to_bits() {centers.push(json!({"input":row["args"][0],
                "direct":format!("0x{:016x}",direct.to_bits()),"reduced":format!("0x{:016x}",reduced.to_bits())}));}
        }
        println!("{}",json!({"searched":capture["witnesses"].as_array().unwrap().len(),"centers":centers}));
        return;
    }
    if std::env::args().any(|a|a=="--selected") {
        let candidates:[(&str,fn(f64)->f64);13]=[
            ("platform-asin",|x|x.asin()),
            ("product-stored-rn64-ratio-fpatan",|x|predict(x,"product","stored-rn64","ratio","fpatan")),
            ("product-stored-rn64-ratio-platform",|x|predict(x,"product","stored-rn64","ratio","platform")),
            ("product-mixed-left-stored",|x|barrier_graph(x,0b11101,false)),
            ("product-mixed-right-stored",|x|barrier_graph(x,0b11110,false)),
            ("complement-stored-rn64-extended",|x|complementary(x,"product-complement","stored-rn64","extended-pi",false)),
            ("reused-minus-binary64-ratio-fpatan",|x|predict(x,"reused-minus","binary64","ratio","fpatan")),
            ("reused-minus-stored-rn64-ratio-fpatan",|x|predict(x,"reused-minus","stored-rn64","ratio","fpatan")),
            ("reused-minus-stored-rn64-ratio-fpatan-reduced",|x|predict(x,"reused-minus","stored-rn64","ratio","fpatan-reduced")),
            ("reused-minus-continuous-rn64-ratio-fpatan",|x|predict(x,"reused-minus","continuous-rn64","ratio","fpatan")),
            ("reused-plus-binary64-ratio-fpatan",|x|predict(x,"reused-plus","binary64","ratio","fpatan")),
            ("reused-plus-stored-rn64-ratio-fpatan",|x|predict(x,"reused-plus","stored-rn64","ratio","fpatan")),
            ("reused-plus-continuous-rn64-ratio-fpatan",|x|predict(x,"reused-plus","continuous-rn64","ratio","fpatan")),
        ];
        for (name,predict) in candidates {
            let mut misses=Vec::new();let mut rows=0;
            for row in capture["witnesses"].as_array().unwrap() {
                let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
                if x.is_subnormal(){continue;}
                let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                    let value=predict(x);
                    let value=if capture["function"]=="ACOS" {
                        if std::env::args().any(|a|a=="--acos-staged-sub") {
                            to(&ext_sub(&from(std::f64::consts::FRAC_PI_2),&from(value),CW),CW)
                        } else {std::f64::consts::FRAC_PI_2-value}
                    } else {value};
                    format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
                };rows+=1;
                if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
            }
            println!("{}",json!({"candidate":name,"rows":rows,"matches":rows-misses.len(),"misses":misses}));
        }
        return;
    }
    if std::env::args().any(|a|a=="--isolate") {
        for row in capture["witnesses"].as_array().unwrap() {
            let n=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            let x=from(n);let one=from(1.0);let s="stored-rn64";
            let product=op(&op(&one,&x,'-',s),&op(&one,&x,'+',s),'*',s);
            let root=sqrt(&product,s);let ratio=op(&x,&root,'/',s);
            println!("{}",json!({"input":row["args"][0],"product":format!("0x{:016x}",to(&product,CW).to_bits()),
                "root":format!("0x{:016x}",to(&root,CW).to_bits()),"ratio":format!("0x{:016x}",to(&ratio,CW).to_bits())}));
        }
        return;
    }
    let mut modes=vec![("platform-asin","binary64","direct","platform")];
    for form in ["difference","product","half-angle","expanded-left","expanded-right"] {
        for stage in ["binary64","stored-rn64","continuous-rn64"] {
            for angle in ["ratio","direct","reciprocal-product"] {for backend in ["platform","platform-atan","fpatan"] {
                modes.push((form,stage,angle,backend));
            }}
        }
    }
    for (form,stage,angle,backend) in modes {
        let mut misses=Vec::new();let mut rows=0;let mut excluded=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){excluded+=1;continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=predict(x,form,stage,angle,backend);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };
            rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("{form}-{stage}-{angle}-{backend}"),"rows":rows,"matches":rows-misses.len(),"input_subnormals_excluded":excluded,"misses":misses}));
    }
    for direct in [false,true] {for mask in 0..32 {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=barrier_graph(x,mask,direct);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("product-rounding-barriers-{mask:05b}-direct-{direct}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
    }}
    for form in ["product-complement","difference-complement","half-angle-complement","inverse-half-angle-complement"] {
      for stage in ["binary64","stored-rn64","continuous-rn64"] {
       for pi_kind in ["binary64-pi","extended-pi"] {for store_angle in [false,true] {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=complementary(x,form,stage,pi_kind,store_angle);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("{form}-{stage}-{pi_kind}-store-angle-{store_angle}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
       }}
      }
    }
    for mask in 0..64 {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=split_root_graph(x,mask);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("split-root-rounding-barriers-{mask:06b}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
    }
    for reverse in [false,true] {for mask in 0..64 {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=split_division_graph(x,mask,reverse);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("split-division-barriers-{mask:06b}-reverse-{reverse}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
    }}
    for mode in 0..243 {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=precision_graph(x,mode);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("product-ternary-precision-{mode:03}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
    }
    for stage in ["binary64","stored-rn64","continuous-rn64"] {
     for pi in ["extended-pi","binary64-pi"] {for stored in [false,true] {
        let mut misses=Vec::new();let mut rows=0;
        for row in capture["witnesses"].as_array().unwrap() {
            let x=f64::from_bits(u64::from_str_radix(row["args"][0].as_str().unwrap().trim_start_matches("0x"),16).unwrap());
            if x.is_subnormal(){continue;}
            let actual=if !(-1.0..=1.0).contains(&x) {"error:Num".to_string()} else {
                let value=reciprocal_angle_graph(x,stage,pi,stored);
                format!("0x{:016x}",if value.abs()<f64::MIN_POSITIVE {0} else {value.to_bits()})
            };rows+=1;
            if actual!=row["expected_bits"].as_str().unwrap(){misses.push(json!({"id":row["id"],"args":row["args"],"expected":row["expected_bits"],"actual":actual}));}
        }
        println!("{}",json!({"candidate":format!("reciprocal-angle-{stage}-{pi}-store-{stored}"),"rows":rows,"matches":rows-misses.len(),"misses":misses}));
     }}
    }
}
