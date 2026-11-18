//! End-to-end tests of the query engine (natural language in, pods out).

use noesis::engine::{content_text, QueryResult};

fn q(input: &str) -> QueryResult {
    noesis::domains::live::set_online(false);
    let r = noesis::query(input);
    assert!(r.success, "{input}: {:?}", r.error);
    r
}

fn short(input: &str) -> String {
    q(input).short_answer().unwrap_or_default()
}

fn pod_text(r: &QueryResult, id: &str) -> String {
    r.pods.iter().find(|p| p.id == id).map(|p| p.subpods.iter().map(|s| content_text(&s.content)).collect::<Vec<_>>().join(" | ")).unwrap_or_else(|| panic!("no pod {id} in {:?}", r.pods.iter().map(|p| &p.id).collect::<Vec<_>>()))
}

#[test]
fn arithmetic_and_numbers() {
    assert_eq!(short("2+2"), "4");
    assert_eq!(short("what is 2^10?"), "1024");
    let r = q("pi");
    assert!(pod_text(&r, "DecimalApproximation").starts_with("3.14159265358979323846"));
    let r = q("100");
    assert_eq!(pod_text(&r, "NumberName"), "one hundred");
    assert_eq!(pod_text(&r, "RomanNumerals"), "C");
    assert!(pod_text(&q("100!"), "ScientificNotation").contains("10^157"));
    assert_eq!(short("(1+2i)^3"), "-11 - 2 i");
}

#[test]
fn calculus_queries() {
    assert_eq!(short("derivative of x^3 ln(x)"), "d/dx(x^3 log(x)) = x^2 (3 log(x) + 1)");
    assert_eq!(short("integrate e^(-x^2) from -inf to inf"), "∫_(-∞)^∞ e^(-x^2) dx = sqrt(π)");
    assert_eq!(short("limit sin(x)/x as x->0"), "lim_(x→0) sin(x)/x = 1");
    assert_eq!(short("sum 1/k^2, k=1 to inf"), "∑_(k=1)^∞ 1/k^2 = π^2/6");
    assert!(short("sum 1/k, k=1 to inf").contains("diverges"));
    assert!(short("integrate 1/x from 0 to 1").contains("does not converge"));
}

#[test]
fn algebra_queries() {
    assert_eq!(short("solve x^3 - 6x^2 + 11x - 6 = 0"), "x = 1 | x = 2 | x = 3".replace(" | ", "\n"));
    assert_eq!(short("factor x^4 - 1"), "(x - 1) (x + 1) (x^2 + 1)");
    assert_eq!(short("x^2 < 4"), "-2 < x < 2");
    assert_eq!(short("{x+y=3, x-y=1}"), "x = 2, y = 1");
    let r = q("x^2+y^2=25");
    assert!(pod_text(&r, "Geometry").contains("circle"));
}

#[test]
fn function_analysis() {
    let r = q("x^2 - 5x + 6");
    assert!(pod_text(&r, "RealRoots").contains("x = 2"));
    assert!(pod_text(&r, "GlobalMinimum").contains("-1/4"));
    assert!(r.pods.iter().any(|p| p.id == "Plot"));
    let r = q("1/x");
    assert!(pod_text(&r, "Properties").contains("x ≠ 0"));
}

#[test]
fn matrices_and_statistics() {
    let r = q("{{1,2},{3,4}}");
    assert_eq!(pod_text(&r, "Determinant"), "-2");
    assert_eq!(pod_text(&r, "Inverse"), "{{-2, 1}, {3/2, -1/2}}");
    let r = q("{1,2,3,4,10}");
    assert!(pod_text(&r, "Statistics").contains("mean | 4"));
}

#[test]
fn units_and_constants() {
    assert_eq!(short("10 miles in km"), "16.09344 kilometers");
    assert_eq!(short("32 F to C"), "0 degrees Celsius");
    assert_eq!(short("5 ft 3 in to cm"), "160.02 centimeters");
    assert_eq!(short("how many feet in a mile"), "5280 feet");
    assert!(short("speed of light").starts_with("299792458 m/s"));
}

#[test]
fn knowledge_domains() {
    assert_eq!(short("molar mass of caffeine"), "194.194 g/mol");
    assert_eq!(short("capital of australia"), "Canberra");
    assert!(short("distance from paris to london").starts_with("343."));
    assert!(pod_text(&q("iron"), "BasicProperties").contains("26"));
    assert_eq!(short("what day was July 4, 1776"), "Thursday, July 4, 1776");
    assert_eq!(short("days between 2024-01-01 and 2024-12-25"), "359 days  |  51 weeks 2 days");
    assert_eq!(short("mortgage $300000 at 6.5% for 30 years"), "$1,896.20");
    assert!(short("probability of 3 heads in 5 coin flips").contains("0.3125"));
}

#[test]
fn multivariable_limits() {
    assert_eq!(short("lim (x,y)->(0,0) y/x"), "The limit does not exist.");
    assert_eq!(short("limit of x y/(x^2+y^2) as (x,y)->(0,0)"), "The limit does not exist.");
    assert_eq!(short("limit x y^2/(x^2+y^4) as (x,y)->(0,0)"), "The limit does not exist.");
    assert_eq!(short("limit x^2 y/(x^2+y^2) as (x,y) -> (0,0)"), "0");
    assert_eq!(short("limit sin(x^2+y^2)/(x^2+y^2) as (x,y)->(0,0)"), "1");
    assert_eq!(short("limit (x^2+y^2) as (x,y)->(1,2)"), "5");
}

#[test]
fn laplace_transforms() {
    assert_eq!(short("laplace transform of t^2 e^t"), "ℒ(e^t t^2) = 2/(s - 1)^3");
    assert_eq!(short("laplace transform of sin(3t)"), "ℒ(sin(3 t)) = 3/(s^2 + 9)");
    assert_eq!(short("inverse laplace transform of 1/(s(s+1))"), "ℒ^(-1)(1/(s (s + 1))) = 1 - e^(-t)");
    assert_eq!(short("inverse laplace of (s+1)/(s^2+2s+5)"), "ℒ^(-1)((s + 1)/(s^2 + 2 s + 5)) = e^(-t) cos(2 t)");
    assert_eq!(short("inverse laplace of 2/(s-3)^3"), "ℒ^(-1)(2/(s - 3)^3) = e^(3 t) t^2");
}

#[test]
fn step_by_step() {
    let r = q("integrate x^2 sin(x) dx");
    let steps = pod_text(&r, "Steps");
    assert!(steps.contains("Integrate by parts"), "{steps}");
    let r = q("derivative of x^3 ln(x)");
    assert!(pod_text(&r, "Steps").contains("product rule"));
    let r = q("x^2 - 5x + 6 = 0");
    assert!(pod_text(&r, "SolutionSteps").contains("quadratic formula"));
    // limits
    let st = pod_text(&q("limit of sin(x)/x as x->0"), "Steps");
    assert!(st.contains("L'Hôpital"), "{st}");
    let st = pod_text(&q("limit (x^2-1)/(x-1) as x->1"), "Steps");
    assert!(st.contains("Cancel the common factor"), "{st}");
    let st = pod_text(&q("limit (1+1/x)^x as x->infinity"), "Steps");
    assert!(st.contains("logarithms"), "{st}");
    let r = q("limit 1/x as x->0");
    assert!(r.pods.iter().any(|p| p.id == "OneSided"));
    // sums
    let st = pod_text(&q("sum k^2 from k=1 to n"), "Steps");
    assert!(st.contains("power-sum"), "{st}");
    let st = pod_text(&q("sum 1/2^k from k=0 to infinity"), "Steps");
    assert!(st.contains("geometric"), "{st}");
    let st = pod_text(&q("sum 1/(k(k+1)) from k=1 to infinity"), "Steps");
    assert!(st.contains("telescopes"), "{st}");
    // equations
    let st = pod_text(&q("solve 2 e^(3x) = 10"), "SolutionSteps");
    assert!(st.contains("Take logarithms"), "{st}");
    let st = pod_text(&q("solve sqrt(x+3) = x - 3"), "SolutionSteps");
    assert!(st.contains("extraneous ✗") && st.contains("valid ✓"), "{st}");
    let st = pod_text(&q("solve x^3 - 6x^2 + 11x - 6 = 0"), "SolutionSteps");
    assert!(st.contains("Factor the polynomial"), "{st}");
}

#[test]
fn differential_equations() {
    assert_eq!(short("y'' + y = 0"), "y(x) = c_1 cos(x) + c_2 sin(x)");
    assert_eq!(short("y' = y, y(0) = 1"), "y(x) = e^x");
    assert_eq!(short("y'' + 4y = sin(x), y(0)=0, y'(0)=1"), "y(x) = (sin(x) + sin(2 x))/3");
    assert_eq!(short("y' + 2y = x"), "y(x) = e^(-2 x) c_1 + x/2 - 1/4");
    assert_eq!(short("y'' + 2y' + y = 0"), "y(x) = e^(-x) (c_2 x + c_1)");
    let r = q("y' = sin(x y), y(0) = 1");
    assert!(r.pods.iter().any(|p| p.id == "NumericSolution"));
}

/// Live data needs the network; run with NOESIS_TEST_NET=1.
#[test]
fn live_data() {
    if std::env::var("NOESIS_TEST_NET").is_err() {
        return;
    }
    noesis::domains::live::set_online(true);
    let r = noesis::query("weather in Paris");
    assert!(r.success && r.pods.iter().any(|p| p.id == "Current"), "{:?}", r.error);
    let r = noesis::query("100 usd to eur");
    assert!(r.short_answer().unwrap_or_default().ends_with("EUR"));
    let r = noesis::query("who was Ada Lovelace");
    assert!(r.short_answer().unwrap_or_default().contains("mathematician"));
}

#[test]
fn unknown_input_fails_gracefully() {
    noesis::domains::live::set_online(false);
    let r = noesis::query("qwzx blorp flim");
    assert!(!r.success);
    assert!(!r.suggestions.is_empty());
}

/// Every query in the natural-language corpus must be understood.
#[test]
fn natural_language_corpus() {
    noesis::domains::live::set_online(false);
    let text = concat!(include_str!("data/corpus.txt"), "\n", include_str!("data/corpus2.txt"), "\n", include_str!("data/corpus3.txt"), "\n", include_str!("data/corpus4.txt"), "\n", include_str!("data/corpus5.txt"), "\n", include_str!("data/corpus6.txt"), "\n", include_str!("data/corpus7.txt"));
    let failed: Vec<&str> = text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).filter(|l| !noesis::query(l).success).collect();
    assert!(failed.is_empty(), "not understood: {failed:#?}");
}

#[test]
fn natural_language_math() {
    assert!(short("1 + 2 + 3 + ... + 100").ends_with("= 5050"));
    assert_eq!(short("absolute value of 3+4i"), "5");
    assert!(short("average value of sin(x) from 0 to pi").contains("2/π"));
    assert_eq!(short("pi to 4 digits"), "3.142");
    assert_eq!(short("e to 10 decimal places"), "2.7182818285");
    assert_eq!(short("sqrt(2) to 30 digits"), "1.41421356237309504880168872421");
    assert!(short("solve x + y = 5 and x - y = 1").contains('3'));
    assert!(short("complete the square x^2 + 6x + 5").contains("(x + 3)"));
    assert!(short("sum of the first 100 odd numbers").ends_with("= 10000"));
    assert!(short("is 28 a perfect number").to_lowercase().contains("yes"));
    assert!(short("determinant of {{1,2},{3,4}}").contains("-2"));
}

#[test]
fn geometry_and_physics_formulas() {
    assert_eq!(short("area of a circle with radius 5"), "25 π");
    assert_eq!(short("hypotenuse of a right triangle with legs 3 and 4"), "5");
    assert_eq!(short("area of a triangle with sides 3, 4, 5"), "6");
    assert_eq!(short("volume of a cone with radius 3 and height 4"), "12 π");
    assert_eq!(short("pi/6 radians in degrees"), "30°");
    assert_eq!(short("kinetic energy of 2 kg at 3 m/s"), "9 J");
    assert_eq!(short("ohm's law 12 V 4 ohm"), "3 A");
    assert!(short("escape velocity of earth").starts_with("11186"));
    assert!(short("how much would 150 lb weigh on the moon").starts_with("24.8"));
    assert!(short("quadratic formula").contains("4 a c"));
    assert!(pod_text(&q("law of cosines a=3 b=4 C=60 degrees"), "Result").contains("3.60555"));
}

#[test]
fn chemistry_calculations() {
    assert_eq!(short("balance C3H8 + O2 -> CO2 + H2O"), "C₃H₈ + 5 O₂ → 3 CO₂ + 4 H₂O");
    assert_eq!(short("balance KMnO4 + HCl -> KCl + MnCl2 + H2O + Cl2"), "2 KMnO₄ + 16 HCl → 2 KCl + 2 MnCl₂ + 8 H₂O + 5 Cl₂");
    assert!(short("pH of 0.01 M HCl").starts_with("2 "));
    assert!(short("pH of 0.1 M acetic acid").starts_with("2.87"));
    assert!(short("pH of 0.05 M NaOH").starts_with("12.7"));
    assert!(short("how many moles in 18 g of water").starts_with("0.999"));
    assert!(short("molecules in 1 g of water").contains("10^22"));
    assert!(short("boiling point of water").starts_with("100 °C"));
    assert!(short("molarity of 5 g of NaCl in 250 mL of water").starts_with("0.342"));
}

#[test]
fn everyday_calculations() {
    assert_eq!(short("30% off 80"), "$56.00");
    assert_eq!(short("simple interest on 500 at 3% for 2 years"), "$30.00");
    assert_eq!(short("present value of 1000 in 5 years at 7%"), "$712.99");
    assert!(short("percent change from 40 to 50").starts_with("+25%"));
    assert_eq!(short("what percent of 80 is 20"), "25%");
    assert!(short("is 2024 a leap year").starts_with("Yes"));
    assert!(short("is 1900 a leap year").starts_with("No"));
    assert_eq!(short("how many days in february 2024"), "29 days");
    assert!(short("unix time 1700000000").contains("November 14, 2023"));
    assert!(short("BMI 70 kg 175 cm").starts_with("22.86"));
    assert_eq!(short("number of letters in mississippi"), "11");
    assert_eq!(short("scrabble score of quiz"), "22 points");
    assert_eq!(short("2 hours 30 minutes in seconds"), "9000 seconds");
    assert_eq!(short("1 hour 15 minutes + 50 minutes"), "2 hours 5 minutes");
    assert_eq!(short("500 calories in kJ"), "2.092 kilojoules");
    assert_eq!(short("sos in morse code"), "... --- ...");
}

#[test]
fn astronomy_and_world_facts() {
    assert!(short("mass of the sun").starts_with("1.98847×10^30 kg"));
    assert!(short("how far is the moon").starts_with("384,400 km"));
    assert!(short("height of mount everest").starts_with("8848.86 m"));
    assert!(short("largest country by area").starts_with("Russia"));
    assert!(short("most populous country").starts_with("India"));
    assert!(short("gdp of germany").contains("trillion"));
    assert!(short("how long is a day on mars").starts_with("24.6"));
}

#[test]
fn more_mathematics_in_words() {
    assert_eq!(short("discriminant of 2x^2 + 3x - 5"), "49");
    assert!(short("is x^2 + 1 irreducible").starts_with("Yes"));
    assert!(short("is x^2 - 1 irreducible").starts_with("No"));
    assert_eq!(short("slope of 3x + 2y = 6"), "-3/2");
    assert_eq!(short("line through (1,2) with slope 3"), "y = 3 x - 1");
    assert!(short("center and radius of x^2 + y^2 - 4x + 6y - 3 = 0").contains("radius 4"));
    assert_eq!(short("implicit derivative of x^2 + y^2 = 1"), "-x/y");
    assert_eq!(short("curvature of y = x^2 at x = 0"), "2");
    assert!(short("integrate x^2 y dx dy x=0..1 y=0..2").ends_with("2/3"));
    assert_eq!(short("solve {{1,2},{3,4}} x = {5,6}"), "x = (-4, 9/2)");
    assert!(short("is {{2,1},{1,2}} positive definite").starts_with("Yes"));
    assert_eq!(short("number of trailing zeros in 100!"), "24");
    assert_eq!(short("perfect numbers below 10000"), "6, 28, 496, 8128");
    assert!(short("is 561 a carmichael number").starts_with("Yes"));
    assert_eq!(short("5 choose 2 with repetition"), "15");
    assert!(short("birthday problem 23 people").starts_with("0.507"));
    assert!(short("chi-square critical value 0.05 df 3").starts_with("7.81"));
    assert!(short("p-value for z = 2.1").contains("0.0357"));
    assert_eq!(short("probability of rolling two sixes"), "1/36");
    assert_eq!(short("probability of 5 heads in a row"), "1/32");
}

#[test]
fn reference_knowledge() {
    assert!(short("noble gases").starts_with("Helium, Neon"));
    assert_eq!(short("oxidation states of manganese"), "-3, -1, 0, +1, +2, +3, +4, +5, +6, +7");
    assert_eq!(short("common oxidation states of manganese"), "+2, +4, +7");
    assert_eq!(short("half-life of carbon-14"), "5,730 years");
    assert_eq!(short("empirical formula of C6H12O6"), "CH2O");
    assert!(short("when is easter 2025").starts_with("Sunday, April 20, 2025"));
    assert!(short("thanksgiving 2025").starts_with("Thursday, November 27, 2025"));
    assert_eq!(short("languages of switzerland"), "German, French, Italian, Romansh");
    assert_eq!(short("md5 of hello"), "5d41402abc4b2a76b9719d911017c592");
    assert_eq!(short("base64 encode hello"), "aGVsbG8=");
    assert!(short("color #ff8800").contains("rgb(255, 136, 0)"));
    assert_eq!(short("angle sum of a hexagon"), "720°");
    assert!(short("force between two 1 C charges 1 m apart").starts_with("8987551792"));
    assert!(short("resistance of 10 ohm and 20 ohm in parallel").starts_with("6.66667"));
    assert_eq!(short("break even fixed costs 5000 variable cost 20 price 45"), "200 units");
    assert!(short("future value of 100 per month at 6% for 20 years").starts_with("$46,204"));
    assert!(short("10 eV in J").starts_with("1.6021766×10^-18"));
    assert!(short("100 usd in eur").starts_with("≈"));
}

#[test]
fn everyday_phrasings() {
    assert_eq!(short("f(x) = x^3 - 3x^2 + 2").contains("f ="), false);
    assert_eq!(short("what is 0.75 as a fraction"), "3/4");
    assert_eq!(short("convert 3/8 to decimal"), "0.375");
    assert!(short("what is 1/7 as a decimal").contains("142857 repeat"));
    assert_eq!(short("how many feet in 3 miles"), "15840 feet");
    assert_eq!(short("how many seconds are in a day"), "86400 seconds");
    assert_eq!(short("what is 3 to the 4th power"), "81");
    assert_eq!(short("what is sin of 45 degrees"), "sqrt(2)/2");
    assert_eq!(short("what is the hypotenuse if the legs are 5 and 12"), "13");
    assert_eq!(short("what is 7 factorial divided by 5 factorial"), "42");
    assert_eq!(short("how many ways can 4 people sit in a row"), "24");
    assert_eq!(short("how many sides does a hexagon have"), "6");
    assert_eq!(short("volume of a 2 m by 3 m by 4 m box"), "24 m³");
    assert_eq!(short("round 2.567 to 1 decimal place"), "2.6");
    assert_eq!(short("how many zeros in a billion"), "9");
    assert_eq!(short("what is 1 million in scientific notation"), "1 × 10^6");
    assert_eq!(short("what is the boiling point of water in fahrenheit"), "212 °F");
    assert_eq!(short("how many calories in a banana"), "105 kcal");
    assert!(short("weight of 1 liter of water").starts_with("0.997 kg"));
    assert_eq!(short("density of gold in kg/m^3"), "19300 kg/m³");
}

#[test]
fn harder_calculus() {
    assert!(short("integrate from 0 to inf x^3 e^(-x) dx").ends_with("= 6"));
    assert!(short("integrate from 0 to inf sin(x)/x dx").ends_with("= π/2"));
    assert!(short("integrate from 0 to pi/2 sin(x)^6 dx").ends_with("= (5 π)/32"));
    assert!(short("integrate from 0 to 1 x ln(x) dx").ends_with("= -1/4"));
    assert!(short("integrate tan(x)^3 dx").contains("tan^2(x)/2 + log(cos(x))"));
    assert_eq!(short("x^2 y'' + x y' - y = 0"), "y(x) = c_2 x + c_1/x");
    assert_eq!(short("x^2 y'' + x y' + y = 0"), "y(x) = c_1 cos(log(x)) + c_2 sin(log(x))");
    assert_eq!(short("x^2 y'' - 2y = x^3"), "y(x) = c_2 x^2 + x^3/4 + c_1/x");
    assert_eq!(short("y' = y(1-y), y(0) = 1/2"), "y(x) = e^x/(e^x + 1)");
    assert_eq!(short("y(t)' = -2 y(t)"), "y(t) = e^(-2 t) c_1");
    assert!(short("integrate (x^2 + 1)/(x^3 + 3x) dx").contains("log(x)/3 + log(x^2 + 3)/3"));
    assert!(short("integrate 1/(x(x+1)) dx").contains("-log(x + 1) + log(x)"));
}

#[test]
fn partial_differential_equations() {
    assert_eq!(short("u_t = 2 u_xx, u(0,t)=0, u(pi,t)=0, u(x,0)=sin(x) + 3 sin(2x)"), "u = 3 e^(-8 t) sin(2 x) + e^(-2 t) sin(x)");
    assert!(short("u_t = u_xx, u(0,t) = 0, u(1,t) = 0, u(x,0) = x(1 - x)").contains("(4 - 4 (-1)^n)"));
    assert!(short("u_t = u_xx, u_x(0,t)=0, u_x(1,t)=0, u(x,0)=x").ends_with("+ 1/2"));
    assert_eq!(short("u_tt = u_xx, u(0,t)=0, u(1,t)=0, u(x,0)=sin(pi x), u_t(x,0)=0"), "u = cos(π t) sin(π x)");
    assert!(short("u_tt = 9 u_xx, u(0,t)=0, u(pi,t)=0, u(x,0)=0, u_t(x,0)=1").contains("sin(3 n t) sin(n x)"));
    assert_eq!(short("u_tt = 4 u_xx, u(x,0) = e^(-x^2), u_t(x,0) = 0"), "u = (e^(-(x - 2 t)^2) + e^(-(2 t + x)^2))/2");
    assert_eq!(short("u_t + 2 u_x = 0, u(x,0) = sin(x)"), "u = -sin(2 t - x)");
    assert_eq!(short("u_xx + u_yy = 0, u(0,y) = 0, u(pi,y) = 0, u(x,0) = 0, u(x,1) = sin(x)"), "u = (sin(x) sinh(y))/sinh(1)");
    assert!(short("u_xx + 2 u_xy + u_yy = 0").starts_with("parabolic"));
    assert!(short("du/dt = 3 d^2u/dx^2, u(0,t)=0, u(2,t)=0, u(x,0)=5").contains("sin((π n x)/2)"));
    assert!(short("wave equation").contains("c²"));
    // source terms and non-zero end values
    assert!(short("u_t = u_xx, u(0,t)=0, u(1,t)=1, u(x,0)=0").starts_with("u = x + "));
    assert!(short("u_t = u_xx + 2, u(0,t)=0, u(1,t)=0, u(x,0)=0").starts_with("u = -x^2 + x + "));
    assert_eq!(short("u_t = u_xx + 2, u(0,t)=0, u(1,t)=0, u(x,0)=x(1-x)"), "u = x - x^2");
    assert!(short("u_tt = u_xx - 2, u(0,t)=0, u(1,t)=0, u(x,0)=0, u_t(x,0)=0").starts_with("u = x^2 - x + "));
    assert_eq!(short("u_t = u_xx + e^(-t) sin(pi x), u(0,t)=0, u(1,t)=0, u(x,0)=0"), "u = (e^(-π^2 t) (e^(t (π^2 - 1)) - 1) sin(π x))/(π^2 - 1)");
}

#[test]
fn advanced_topics() {
    assert_eq!(short("solve x^2 = 2 mod 7"), "x ≡ 3,  x ≡ 4  (mod 7)");
    assert_eq!(short("solve 3x = 4 mod 7"), "x ≡ 6  (mod 7)");
    assert_eq!(short("inverse of 3 mod 11"), "4");
    assert_eq!(short("pell equation x^2 - 2y^2 = 1"), "x = 3, y = 2");
    assert_eq!(short("partitions of 10"), "42");
    assert_eq!(short("collatz 27"), "111");
    assert!(short("bessel j0(2.5)").starts_with("-0.04838"));
    assert!(short("elliptic integral K(0.5)").starts_with("1.854074677"));
    assert!(short("hypergeometric 2F1(1,1,2,0.5)").starts_with("1.386294361"));
    assert_eq!(short("matrix exponential of {{0,1},{-1,0}}"), "{{cos(1), sin(1)}, {-sin(1), cos(1)}}");
    assert!(short("singular values of {{3,0},{4,5}}").contains("3 sqrt(5)"));
    assert!(short("solve x' = y, y' = -x").contains("x(t) = c_1 cos(t) + c_2 sin(t)"));
    assert!(short("recurrence a(n) = a(n-1) + a(n-2), a(0)=0, a(1)=1").contains("sqrt(5)"));
    assert_eq!(short("z transform of 2^n"), "z/(z - 2)");
    assert_eq!(short("inverse z transform of z/(z-1)"), "f(n) = 1");
    assert_eq!(short("convolution of e^t and e^(2t)"), "e^(2 t) - e^t");
    assert_eq!(short("residue of 1/(z^2+1) at z = i"), "-i/2");
    assert_eq!(short("contour integral of 1/z around unit circle"), "2 π i");
    assert!(short("fourier transform of e^(-x^2)").contains("e^(-ω^2/4)"));
    assert!(short("product (1 - 1/k^2) from k=2 to infinity").ends_with("= 1/2"));
    assert!(short("product k^2/(k^2 - 1/4) from k=1 to infinity").ends_with("= π/2"));
    assert!(short("sum (-1)^n/(2n+1) from n=0 to infinity").ends_with("= π/4"));
    assert!(short("sum (-1)^(n+1)/n^4 from n=1 to infinity").ends_with("= (7 π^4)/720"));
}

#[test]
fn space_curves() {
    let r = q("parametric plot (cos t, sin t, t)");
    assert_eq!(pod_text(&r, "Length"), "4 sqrt(2) π");
    let p = r.pods.iter().find(|p| p.id == "Plot").unwrap();
    match &p.subpods[0].content {
        noesis::engine::Content::Plot { plot: pl } => {
            assert_eq!(pl.kind, noesis::plot::PlotKind::Curve3d);
            assert!(noesis::plot::to_svg(pl, true, 640.0, 400.0).contains("<polyline"));
        }
        _ => panic!("no plot"),
    }
}

#[test]
fn harder_integrals() {
    assert!(short("integrate 1/(x^4+1) dx").contains("tan^(-1)(sqrt(2) x + 1)"));
    assert!(short("integrate 1/(x^6+1) dx").contains("log(x^2 + sqrt(3) x + 1)"));
    assert!(short("integrate from 0 to inf 1/(1 + x^4) dx").ends_with("= (sqrt(2) π)/4"));
    assert!(short("integrate from 0 to inf sin(x^2) dx").ends_with("= (sqrt(2) sqrt(π))/4"));
    assert!(short("integrate from 0 to 2 pi 1/(2 + cos(x)) dx").starts_with("∫_0^(2 π)"));
    assert_eq!(pod_text(&q("integrate from 0 to pi/2 ln(sin(x)) dx"), "ClosedForm"), "-(π log(2))/2");
    assert_eq!(pod_text(&q("integrate from 0 to inf x/(e^x - 1) dx"), "ClosedForm"), "π^2/6");
    assert_eq!(pod_text(&q("integrate 1/(1+sin(x)^2) from 0 to pi"), "ClosedForm"), "(sqrt(2) π)/2");
    assert!(!q("integrate from 0 to 1 x^x dx").pods.iter().any(|p| p.id == "ClosedForm"));
}

#[test]
fn course_topics() {
    assert_eq!(short("directional derivative of x^2 + y^2 at (1,1) in direction (1,0)"), "2");
    assert_eq!(short("tangent plane to z = x^2 + y^2 at (1,1)"), "z = 2 x + 2 y - 2");
    assert_eq!(short("lagrange multipliers maximize xy subject to x + y = 10"), "maximum 25 at (5, 5)");
    assert_eq!(short("interval of convergence of sum x^n/n"), "[-1, 1)");
    assert_eq!(short("riemann sum of x^2 from 0 to 1 with 4 rectangles"), "0.21875");
    assert_eq!(short("trapezoidal rule for x^2 from 0 to 2 with 4 intervals"), "2.75");
    assert!(short("newton's method for x^2 - 2 starting at 1").starts_with("1.41421356237"));
    assert_eq!(short("euler's method y' = y, y(0) = 1, h = 0.1, to x = 0.5"), "1.61051");
    assert_eq!(short("volume of solid of revolution y = x^2 about x-axis from 0 to 1"), "π/5");
    assert_eq!(short("centroid of region under y = x^2 from 0 to 1"), "(3/4, 3/10)");
    assert_eq!(short("area of a polar curve r = 2 cos(theta)"), "π");
    assert_eq!(short("geometric distribution p = 0.2 P(X = 3)"), "0.128");
    assert!(short("hypergeometric distribution N=50 K=5 n=10 k=2").starts_with("0.20983971"));
    assert!(short("chi square test 10 20 30 vs 20 20 20").starts_with("χ² = 10"));
    assert_eq!(short("sample size for margin of error 0.03 at 95% confidence"), "1068 (worst case p = 0.5)");
}

#[test]
fn matrix_arithmetic_like_wolframalpha() {
    assert_eq!(short("{{1,2},{3,4}}^2"), "{{7, 10}, {15, 22}}");
    assert_eq!(short("{{1,2},{3,4}}^-1"), "{{-2, 1}, {3/2, -1/2}}");
    assert_eq!(short("2 {{1,2},{3,4}}"), "{{2, 4}, {6, 8}}");
    assert_eq!(short("1/10 {{6, -7}, {-2, 4}}"), "{{3/5, -7/10}, {-1/5, 2/5}}");
    assert_eq!(short("{{1,2},{3,4}}^2 - 5 {{1,2},{3,4}}"), "{{2, 0}, {0, 2}}");
    assert_eq!(short("({{1,2},{3,4}}) - ({{1,1},{1,1}})"), "{{0, 1}, {2, 3}}");
}

#[test]
fn words_and_numbers_from_the_benchmark() {
    assert_eq!(short("anagrams of listen"), "elints  |  enlist  |  inlets  |  intels  |  silent  |  tinsel");
    assert!(short("definition of serendipity").contains("good luck in making unexpected and fortunate discoveries"));
    assert_eq!(short("words matching c_t"), "cat  |  cit  |  cot  |  cut");
    assert!(short("is 1729 a taxicab number").starts_with("Yes"));
    assert_eq!(short("100th digit of pi"), "9");
    assert_eq!(short("integrate sqrt(1 + x^4) dx"), "∫ sqrt(x^4 + 1) dx = x ₂F₁(-1/2, 1/4, 5/4, -x^4) + constant");
    assert!(short("mass of the moon").starts_with("7.3458×10^22 kg"));
    // general knowledge from the bundled Wikidata extract
    assert_eq!(short("who wrote hamlet"), "William Shakespeare");
    assert_eq!(short("who composed the magic flute"), "Wolfgang Amadeus Mozart");
    assert_eq!(short("who directed jaws"), "Steven Spielberg");
    assert_eq!(short("when did world war 2 end"), "Sunday, September 2, 1945");
    assert!(short("who was the first president of the united states").starts_with("George Washington"));
    assert_eq!(short("nobel prize in physics 1921"), "Albert Einstein");
    // live positions: the Moon is always between perigee and apogee
    let d: f64 = short("distance from earth to moon").split(" km").next().unwrap().replace(',', "").parse().unwrap();
    assert!((356_000.0..407_000.0).contains(&d), "{d}");
}

#[test]
fn questions_from_the_desktop_app() {
    // phrasings typed into the app that were not understood
    for q in ["the limit of x as x approaches 0 for 1/x", "the limit as x approaches 0 of 1/x", "the limit as x>0 for 1/x"] {
        assert_eq!(short(q), "The two-sided limit does not exist: the one-sided limits differ.", "{q}");
    }
    assert_eq!(short("the limit as x approaches infinity of (1+1/x)^x"), "lim_(x→∞) (1/x + 1)^x = e");
    assert_eq!(short("1+1="), "2");
    assert_eq!(short("1^infinity"), "indeterminate");
    assert_eq!(short("(sin(x))^cos(x)=2"), "x = 2 π n + 2.66536, n ∈ ℤ");
    assert!(short("integrate sin(x)^e^e").contains("₂F₁(1/2, (1 - e^e)/2, 3/2, cos^2(x))"));
}


#[test]
fn graphs_sequences_and_poles() {
    assert_eq!(short("matching polynomial of the Petersen graph"), "x^10 − 15x^8 + 75x^6 − 145x^4 + 90x^2 − 6");
    assert_eq!(short("1->2, 2->3, 3->1 eulerian cycle"), "1 → 2 → 3 → 1");
    assert!(short("(10,8) cage graph").starts_with("1640 vertices"));
    assert_eq!(short("1,000,000th prime"), "15485863");
    assert_eq!(short("1000th twin prime"), "(79559, 79561)");
    assert_eq!(short("sum of first five square triangular numbers"), "1456599");
    assert_eq!(short("is 4 a member of the superperfect numbers?"), "True");
    assert_eq!(short("csc^7(z) residue at z=0"), "5/16");
    assert_eq!(short("poles of 1/sin(z)^2"), "pole | order | residue\nπ n | 2 | 0");
    assert!(short("residues of 1/(e^(2z)+1) with |z| < 3pi").contains("(5 π i)/2 | 1 | -1/2"));
}

#[test]
fn school_calculus_and_word_problems() {
    assert_eq!(short("Rachel has 17 apples. She gives 9 to Sarah. How many apples does Rachel have now?"), "8 apples");
    assert_eq!(short("if there are 40 cookies all together and A takes 10 and B takes 5 how many are left"), "25");
    assert_eq!(short("If Jane has 23 cats and I have 2 cats, and then Jane gives me 5 cats, how many more cats does Jane have than I?"), "11");
    assert_eq!(short("Z transform of n^4"), "(z (z^3 + 11 z^2 + 11 z + 1))/(z - 1)^5");
    assert_eq!(short("FT sinc t"), "ℱ(ω) = (sqrt(2) sqrt(π) θ(1 - |ω|))/2");
    assert_eq!(short("1/2^2 + 1/3^2 + 1/5^2 + 1/7^2 + ..."), "P(2) ≈ 0.452247420041");
    assert_eq!(short("sum n^(-2)"), "∑_(n=1)^∞ 1/n^2 = π^2/6");
    assert_eq!(short("normal to 3x^2-2xy+y^2=1 at x=0, y=1"), "y = 1 - x");
    assert_eq!(short("period of 6 sin(x/3) - 7"), "6 π ≈ 18.8496");
    assert_eq!(short("if sin(a) = 3/5, what is cos(a)?"), "±4/5");
    assert_eq!(short("7.5% of $12.95"), "$0.97 (exactly $0.97125)");
    assert!(short("maximize xyz in x^2+2y^2+3z^2<=1").starts_with("maximum 0.0785674"));
}

#[test]
fn probability_and_random_variables() {
    let a = |s: &str| short(s);
    assert!(a("probability of 3rd head on 8th flip").contains("21/256"));
    assert!(a("X~Poisson(7.3), EV[3X^4-7]").contains("166557683/10000"));
    assert!(a("prob X<16 for X~geometric with p=0.1").contains("0.814697"));
    assert!(a("3rd raw moment of the normal distribution").contains("3 μ σ^2"));
    assert!(a("run of 3 heads in 15 flips").contains("22159/32768"));
    assert!(a("X~N(1,2), P(X>3)").contains("0.158655"));
}

#[test]
fn molecular_graphs() {
    assert_eq!(short("cyclopentane Schultz index"), "80");
    assert_eq!(short("how many h-bond acceptors are in caffeine"), "3");
    assert!(short("butane topological indices").contains("Wiener index | 10"));
    assert!(short("trimethylamine hybridization").contains("sp³ | 3 C, 1 N"));
    assert!(short("what are the bond energies in acetonitrile").contains("C≡N | 1 | 891 kJ/mol"));
    assert_eq!(short("margaric acid carbon backbone"), "17 atoms");
}

#[test]
fn physical_chemistry() {
    assert!(short("boiling point of benzene at 550 torr").starts_with("69.95"));
    assert!(short("vapor pressure of water at 50 C").starts_with("12.30"));
    assert!(short("titration of 50 mL of 2M of strong acid with 3M of strong base").starts_with("33.3333 mL"));
    assert_eq!(short("ice at -78C, 2 atm"), "solid (ice Ih)");
    assert!(short("Goff-Gratch equation for water at 50 F").starts_with("12.27"));
    assert_eq!(short("Write the short electronic configuration for Sn"), "[Kr] 4d10 5s2 5p2");
}

#[test]
fn units_and_standards() {
    assert!(short("20mL in drams").starts_with("5.41024"));
    assert!(short("(20 feet)(1 mile) in acres").starts_with("2.42424"));
    assert_eq!(short("how many ounces in a cup?"), "8 US fluid ounces");
    assert!(short("1 million miles").starts_with("1609344000"));
    assert!(short("convert 120 km/h to imperial").starts_with("74.5645"));
    assert!(short("175lb vs 100kg").starts_with("100 kg is 1.25978×"));
    assert_eq!(short("thickness of steel gauge #11 in mm"), "3.0378 mm");
    assert_eq!(short("sieve size 500"), "25 μm");
    assert!(short("NPS 1 inch steel 25ft").starts_with("19.059 kg"));
    assert!(short("gas mark 1/4").starts_with("225 °F"));
}

#[test]
fn number_names_types_and_wigner_symbols() {
    assert_eq!(short("spell 40"), "forty");
    assert_eq!(short("1 vigintillion"), "10^63");
    assert_eq!(short("five sixteenths"), "5/16");
    assert_eq!(short("integer / integer"), "rational number");
    assert_eq!(short("negative integer - nonnegative rational"), "negative rational number");
    assert_eq!(short("wigner 3j (1, 1, 0, 0, 0, 0)"), "-sqrt(3)/3");
    assert_eq!(short("clebsch gordan 1/2 1/2 1/2 -1/2 1 0"), "sqrt(2)/2");
}

#[test]
fn periodic_and_substitution_tilings() {
    assert!(pod_text(&q("snub square tiling"), "Properties").contains("3.3.4.3.4"));
    assert!(pod_text(&q("kagome tiling"), "Properties").contains("rhombille"));
    assert!(pod_text(&q("truncated trihexagonal tiling"), "Properties").contains("4.6.12"));
    assert!(pod_text(&q("pinwheel tiling 4 steps"), "Properties").contains("625"));
    assert!(pod_text(&q("domino tiling 3 steps"), "Properties").contains("64"));
    assert!(pod_text(&q("penrose tiling"), "Properties").contains("thick"));
}

#[test]
fn kinetic_theory_and_cosmology() {
    assert!(pod_text(&q("msd for neon at 800 K"), "Result").contains("811.9"));
    assert!(pod_text(&q("rms speed of nitrogen at 25 °C"), "Result").contains("515.2"));
    assert!(pod_text(&q("Bose-Einstein condensation temperature Rb"), "Result").contains("398.3"));
    assert!(pod_text(&q("1 billion years after the big bang"), "Result").contains("5.66"));
    assert!(pod_text(&q("age of the universe at z=1100"), "Result").contains("thousand years"));
}

#[test]
fn finance_calculators() {
    assert_eq!(short("continuous interest PV=$3800, rate=6%, periods=5"), "$5,129.46");
    assert_eq!(short("$1000 at 7% for 3 years"), "$1,225.04");
    assert_eq!(short("bond pricing formula $1000, coupon rate 7%, annual yield 10%"), "$815.66");
    assert_eq!(short("fuel cost 50 miles, 20 mpg, $2.09/gal"), "$5.23");
    assert_eq!(short("$10.25/hr"), "$21,320.00 per year");
    assert_eq!(short("cpm 50 imp/s 12000 dollars per day"), "$2.78");
    assert_eq!(short("marginal tax rate of filing single earning $40k"), "12%");
    assert!(short("credit card balance $9000, apr 12%, $300/month").starts_with("36 months"));
}

#[test]
fn health_calculators() {
    assert!(short("BAC 5 drinks, 2 hours, male, 180lb").starts_with("0.0961%"));
    assert!(short("blood pressure 145/90").starts_with("stage 2"));
    assert!(short("due date, last period=3/14/09").contains("December 19, 2009"));
    assert!(pod_text(&q("20/50 vision"), "Equivalents").contains("0.4"));
}

#[test]
fn atmosphere_earth_and_materials() {
    assert!(short("atmosphere 40000ft").starts_with("18.82"));
    assert!(short("speed of 1mm raindrop").starts_with("3.997"));
    assert!(short("3000 miles below the surface of the earth").starts_with("outer core"));
    assert_eq!(short("Vickers hardness formula 30 kgf, 0.5 mm"), "222.528 HV");
    assert_eq!(short("lumber volume 3 ft diameter, 20 ft length"), "1280 board feet");
}

#[test]
fn lighter_queries() {
    assert_eq!(short("hue_36 to base 29"), "red_29");
    assert_eq!(short("warp factor 6"), "392.498 times the speed of light");
    assert_eq!(short("what is warp 9.9 in the next generation"), "3053 times the speed of light");
    assert_eq!(short("What is Wolfram|Alpha spelled backwards?"), "ahplA|marfloW");
    assert!(short("look and say sequence").starts_with("1, 11, 21, 1211, 111221"));
    assert!(short("20 lb turkey frying time").starts_with("60–100 minutes"));
    assert!(short("How many astronomers does it take to change a light bulb?").contains("dark"));
}

#[test]
fn number_theory_numerals_and_triangles() {
    assert_eq!(short("last digit of 9^9^9"), "9");
    assert_eq!(short("last 4 digits of 2^3^4"), "2352");
    assert_eq!(short("solve 2x = 10 (mod 12), 3x = 9 (mod 12)"), "x ≡ 11 (mod 12)");
    assert_eq!(short("number of primitive polynomials of GF(3125)"), "280");
    assert_eq!(short("MMVIII - 25"), "MCMLXXXIII");
    assert_eq!(short("1275 to Greek numerals"), "͵ΑΣΟΕʹ");
    assert_eq!(short("1000 to Babylonian"), "16 40");
    assert_eq!(short("49 tredecillion"), "4.9×10^43");
    assert_eq!(short("silver ratio"), "1 + √2");
    assert_eq!(short("Is the golden ratio transcendental?"), "no");
    assert!(short("law of sines a=6cm, b=8cm, alpha=40deg").contains("β = 121.013°"));
    assert!(short("is f(x,y,z)=sin(xyz) an odd function?").contains("odd function"));
}

#[test]
fn everyday_objects_and_estimates() {
    assert_eq!(short("how many liters in 6 cans of coke?"), "2.13 liters");
    assert_eq!(short("football field length in inches"), "4320 inches");
    assert_eq!(short("how many stripes on the American flag?"), "13");
    assert_eq!(short("how many legs on a spider vs legs on a cow?"), "spider: 8, cow: 4");
    assert_eq!(short("How many m&m's would fit in 2 liters?"), "about 2000");
    assert!(short("how heavy is an elephant?").starts_with("6000 kg"));
    assert!(short("compare lengths of a t-rex and a triceratops").starts_with("3.3 m"));
}

#[test]
fn ranges_concavity_and_rms() {
    assert_eq!(short("range of 1/sqrt(x^2+1)"), "{y ∈ ℝ : 0 < y ≤ 1}");
    assert_eq!(short("range of arctan(x)"), "{y ∈ ℝ : -π/2 < y < π/2}");
    assert_eq!(short("range of x^2-x"), "{y ∈ ℝ : y ≥ -1/4}");
    assert_eq!(short("concavity of the curve xe^(-x)"), "concave down on (−∞, 2); concave up on (2, ∞)");
    assert_eq!(short("concavity of ln(x)"), "concave down on (0, ∞)");
    assert_eq!(short("monotonicity of x^3-3x"), "increasing on (−∞, -1); decreasing on (-1, 1); increasing on (1, ∞)");
    assert!(short("root mean square 3sin(t)-2cos(2t)").starts_with("√(13/2) ≈ 2.5495"));
}

#[test]
fn weather_driving_dates_and_packing() {
    assert_eq!(short("wind chill calculator, 25 degrees F, 30mph"), "8 °F  (-13.3 °C)");
    assert_eq!(short("frostbite, -10 degrees F, 25 mph"), "frostbite possible in about 10 minutes");
    assert_eq!(short("stopping distance 70mph"), "388 ft  (118.3 m)");
    assert_eq!(short("3rd wedding anniversary"), "leather");
    assert_eq!(short("20 million years ago"), "Miocene epoch, Neogene period, Cenozoic era, Phanerozoic eon");
    assert!(short("pack 24 2\" circles in a square").starts_with("smallest square side ≈ 9.86"));
    assert!(short("pack 24 circles in a circle").starts_with("smallest circle diameter ≈ 5.65"));
    assert!(short("next month").contains(' '));
    assert!(short("days until Labor Day").contains("days"));
}

#[test]
fn clocks_calendars_and_holidays() {
    assert_eq!(short("third Tuesday in April 2027"), "Tuesday, April 20, 2027");
    assert_eq!(short("29th Sunday in 2025"), "Sunday, July 20, 2025");
    assert_eq!(short("If it is 7pm in Moscow, what time is it in Islamabad?"), "9:00 PM PKT");
    assert_eq!(short("Mar 4, 1991 to Sep 7, 1995"), "4 years, 6 months, 3 days");
    assert_eq!(short("convert Julian calendar February 9, 2012 to Gregorian calendar"), "Wednesday, February 22, 2012");
    assert_eq!(short("2:34:06pm May 12, 1979 in Unix Time"), "295367646");
    assert!(short("Rosh Hashanah 2015").contains("September 14, 2015"));
    assert!(short("holidays 2009").contains("Thursday, November 26"));
    assert!(short("year 1951").contains("Metal Rabbit"));
}

#[test]
fn inflation_sizes_threads_and_moist_air() {
    assert_eq!(short("US$250 1965 dollars in 1995"), "$1,209.52");
    assert_eq!(short("inflation from 1980 to 2020"), "+214.09% cumulative (2.9% per year)");
    assert_eq!(short("US women's size 6 shoe in french shoe size"), "36.5 (EU / French)");
    assert_eq!(short("U.K. men's size 11 shoe in Japanese size"), "30 cm (Japan / Mondopoint 300 mm)");
    assert_eq!(short("What is the thread count of a coarse #6 bolt?"), "32 threads per inch");
    assert!(short("1/2 inch bolt").starts_with("1/2-13 UNC"));
    assert!(short("lumber volume diameter=3 ft, length=20 ft").contains("1280 board feet"));
    assert!(short("moist air drybulb 72F, wetbulb 66F, pressure 29inHg").starts_with("relative humidity 73"));
    assert_eq!(short("dew point of 25C and 50% humidity"), "dew point 13.9 °C (57 °F)");
    assert!(short("sound speed of air -40 F").starts_with("306.1 m/s"));
}

#[test]
fn lagrangians_waveguides_and_flow_formulas() {
    let a = q("classical harmonic oscillator in 2D in polar coordinates");
    assert!(pod_text(&a, "EOM").contains("θ'' = -(2 r' θ')/r"));
    assert!(pod_text(&a, "Conserved").contains("p_θ = m r^2 θ'"));
    let a = q("classical inverse-distance potential in 3D in spherical coordinates");
    assert!(pod_text(&a, "Lagrangian").contains("+ k/r"));
    assert!(short("Lagrangian of a coupled pendulum system").starts_with("L = ½ m L²"));
    assert!(short("rectangular waveguide a=2.286 cm b=1.016 cm").starts_with("6.557"));
    assert!(short("circular waveguide radius 1 cm").starts_with("8.78"));
    assert!(short("venturi meter inlet diameter 10 cm throat diameter 5 cm pressure difference 20 kPa").starts_with("0.01282"));
    assert!(short("Rossiter formula for mach 2").starts_with("0.1997"));
    assert!(short("radioactive decay, half-life 5 days, time 2 days").starts_with("0.7578"));
}

#[test]
fn restricted_ranges_saddles_and_unit_arithmetic() {
    assert_eq!(short("range of 1/sqrt(x^2+1) restricted to 1<x<4"), "{y ∈ ℝ : 1/√17 < y < 1/√2}");
    assert_eq!(short("range of 1/x restricted to x>1"), "{y ∈ ℝ : 0 < y < 1}");
    assert_eq!(short("domain of f(x,y) = log(1-(x^2+y^2))"), "{(x, y) ∈ ℝ² : x^2 + y^2 < 1}");
    assert_eq!(short("period of sin(x y)"), "x: 2π/y (for y ≠ 0); y: 2π/x (for x ≠ 0)");
    assert!(short("rotate 30 degrees").starts_with("( sqrt(3)/2  -1/2 )"));
    assert_eq!(short("fractional derivative x^2 {x,1/2}"), "(8 x^(3/2))/(3 sqrt(π))");
    assert_eq!(short("cusps of sqrt |x-2| - cbrt |x+2|"), "(-2, 2), (2, -1.5874)");
    assert!(short("area between y=sinc(x) and the x-axis from x=-4pi to 4pi").starts_with("5.449"));
    assert_eq!(short("saddle point of sin(x y)-tan x near (5,6)"), "saddle point at (x, y) ≈ (0, 1), value 0");
    assert_eq!(short("1 + (even number * odd number)"), "odd integer");
    assert_eq!(short("negative number ^ 40"), "positive real number");
    assert_eq!(short("transfer function (s+1)/(s^2+3s+2)"), "stable: every pole has negative real part");
    assert_eq!(short("(5 meters - 2 meters)/(18 seconds - 8 seconds)"), "0.3 m/s");
    assert_eq!(short("solve y'(x) = -2 y+x apply heun method"), "0.42181");
}

#[test]
fn eclipses_stars_and_spaceflight() {
    assert!(short("solar eclipses in 2027").starts_with("February 6, 2027 (annular solar); August 2, 2027 (total solar)"));
    assert_eq!(short("lunar eclipses in 2026"), "March 3, 2026 (total lunar); August 28, 2026 (partial lunar)");
    assert!(short("moon phase on July 4 2026").starts_with("waning gibbous"));
    assert_eq!(short("apparent magnitude of Rigel"), "0.13");
    assert!(short("angular separation between Mizar and Alcor").starts_with("11.8"));
    assert!(short("the shortest distance of the Earth to our Sun during orbit").starts_with("0.9833 au"));
    assert!(short("mass of our Moon vs. Phobos vs. Deimos").contains("Phobos: 1.0659×10^16 kg"));
    assert!(short("When can I see Orion?").starts_with("January"));
    assert_eq!(short("100 year anniversary of the discovery of Pluto"), "Monday, February 18, 2030");
    assert!(short("Sputnik launch date to Sputnik decay date").starts_with("92 days"));
    assert!(short("first animal in orbit").starts_with("Laika"));
}

#[test]
fn materials_food_formulas_and_inequalities() {
    assert!(short("compare thermal conductivity of copper, aluminum, iron, carbon steel, stainless steel").starts_with("Copper: 401 W/(m K) | Aluminum: 237"));
    assert_eq!(short("density of mahogany, walnut wood"), "Mahogany: 0.59 g/cm³ (36.83 lb/ft³) | Black walnut: 0.61 g/cm³ (38.08 lb/ft³)");
    assert_eq!(short("Mohs 6.5"), "between orthoclase feldspar (6) and quartz (7)");
    assert_eq!(short("hardness of iron vs copper"), "Iron: 4 | Copper: 3");
    assert_eq!(short("stitches on a baseball"), "108");
    assert!(short("Calculate Laplace number, 0.05 N/m, 1420 kg/m^3, 0.01m, 10680 cP").starts_with("0.00622"));
    assert_eq!(short("Pasteur units for beer, 60°C, 20 min"), "20");
    assert!(short("1m 1A Helmholtz coil").starts_with("8.99176×10^-7 T"));
    assert_eq!(short("10680 cP in Pa s"), "10.68 pascals seconds");
    assert_eq!(short("9x - 7i > 3 (3x - 7u) solve for i"), "i < 3 u");
    assert_eq!(short("round π% of 1337"), "42");
    assert_eq!(short("peak zenith hourly rate Quadrantids / Eta Aquariids"), "Quadrantids: 110 per hour | Eta Aquariids: 50 per hour");
}

#[test]
fn holomorphic_functions_quadrics_and_state_space() {
    assert_eq!(short("caputod exp(t) {t, 1/7}"), "e^(t) (1 − Γ(6/7, t)/Γ(6/7))");
    assert_eq!(short("holomorphic function imaginary part Sinh[x] Sin[y]"), "f(z) = cosh(z) + c, with c real");
    assert_eq!(short("limit arctan(x^2/y^2) as x -> 0 as y -> 0"), "0");
    assert_eq!(short("differential equations sin 2x"), "y″ + 4 y = 0");
    assert_eq!(short("differential equations J_2(x)"), "x² y″ + x y′ + (x² − 4) y = 0");
    assert_eq!(short("plot x^2 - 3y^2 - z^2 = 1"), "x^2 - 3 y^2 - z^2 = 1 is a hyperboloid of two sheets");
    assert_eq!(short("state {{0,1,0},{0,0,1},{1/5,-1,0}}, input {{0},{0},{1}}, output {{-3,0,1}}"), "H(s) = (s^2 - 3)/(s^3 + s - 1/5)");
    assert!(short("julia set -1").starts_with("connected"));
    assert!(short("random permutation on 15 elements").starts_with('['));
    assert_eq!(short("2 by 3 rectangle rotated 45 degrees"), "(0, 0), (sqrt(2), sqrt(2)), (-sqrt(2)/2, (5 sqrt(2))/2), (-(3 sqrt(2))/2, (3 sqrt(2))/2)");
    assert_eq!(short("what is the outlier for the data set 21.3, 22, 12.7, 11, 100"), "100");
    assert_eq!(short("density of wet sand"), "1.92 g/cm³  (1920 kg/m³)");
}

#[test]
fn positions_seawater_codes_and_bonds() {
    assert_eq!(short("weight of 1 cup milk in grams"), "243.686 grams  (0.243686 kg  (0.537235 lb))");
    assert!(short("55 deg 45' N, 37 deg 37' E").contains("Moscow"));
    assert!(short("23.5S 46.4W to 56.8N 60.6E").starts_with("13201 km"));
    assert!(short("Paris to Berlin to Rome").starts_with("2062.6 km"));
    assert!(short("seawater with 33 psu at 4 C").contains("1026.195 kg/m³"));
    assert!(short("barometric pressure at extreme altitudes").starts_with("31.44 kPa"));
    assert_eq!(short("codes that can correct 7 errors"), "minimum Hamming distance d ≥ 15");
    assert_eq!(short("golay code"), "[23, 12, 7]");
    assert_eq!(short("information rate of BCH code 31, 5"), "11/31 ≈ 0.3548");
    assert_eq!(short("N N bonds"), "N–N: 163 kJ/mol, 145 pm | N=N: 418 kJ/mol, 125 pm | N≡N: 945 kJ/mol, 110 pm");
    assert_eq!(short("MOM ether functional group"), "R–O–CH₂–O–CH₃");
    assert!(short("cosmic origins of gold and iron").starts_with("Gold: merging neutron stars"));
    assert!(!noesis::query("salt to sugar").pods.iter().any(|p| p.id == "Route" || p.title == "Distance"));
}

#[test]
fn animals_taxa_and_breeds() {
    assert!(short("jaguar").starts_with("Panthera onca"));
    assert_eq!(short("how fast can a giraffe run?"), "60 km/h (37.28 mph)");
    assert!(short("air speed of a peregrine falcon vs land speed of a cheetah").contains("cheetah: 112 km/h"));
    assert_eq!(short("wingspan of the greater flamingo"), "1.4–1.65 m (4.6–5.4 ft)");
    assert_eq!(short("average lifespan of a horse, goat, and sheep"), "horse: 25–30 years | goat: 15–18 years | sheep: 10–12 years");
    assert_eq!(short("taxonomy of the great white shark"), "Animalia > Chordata > Chondrichthyes > Lamniformes > Lamnidae > Carcharodon carcharias");
    assert!(short("Lepidoptera").starts_with("butterflies and moths"));
    assert!(short("tallest dog breeds").starts_with("Irish Wolfhound"));
    assert_eq!(short("goats with female height less than 23 inches"), "Pygmy | Nigerian Dwarf");
    assert_eq!(short("cat breeds with mean weight < 8 lbs"), "Singapura | Devon Rex | Cornish Rex | Turkish Angora");
    assert!(short("most milk producing cow").starts_with("Holstein Friesian"));
    assert_eq!(short("Is a British Longhair cat > a Pomeranian's weight?"), "True");
    assert_eq!(short("lifespan of a pug"), "12–15 years");
    assert!(short("origin of Targhee sheep").starts_with("United States"));
    assert!(short("dog").starts_with("Canis familiaris"));
    assert!(!noesis::query("swift").pods.iter().any(|p| p.subpods.iter().any(|s| noesis::engine::content_text(&s.content).contains("Apus"))));
}

#[test]
fn mythology() {
    assert_eq!(short("Zeus symbols"), "thunderbolt, eagle, bull, oak tree, scepter, aegis");
    assert!(short("Where was Zeus born?").starts_with("born in a cave on Mount Ida"));
    assert_eq!(short("What culture does Theseus come from?"), "Greek (Athenian) mythology");
    assert!(short("what else was the god Jupiter called?").starts_with("Jove"));
    assert_eq!(short("compare the symbols of Fortuna and Nike"), "Fortuna: wheel of fortune, cornucopia, rudder, globe | Nike: wings, laurel wreath, palm branch");
    assert!(short("Ānanda's special ability").starts_with("his exceptional memory"));
    assert!(short("Assyrian sphinx vs Egyptian sphinx").starts_with("Assyrian sphinx: protection"));
    assert_eq!(short("what is Thoth's gender?"), "male");
    assert!(short("thor").starts_with("Norse god"));
    // planets keep their usual meaning
    assert!(!short("venus").contains("goddess"));
}

#[test]
fn structures_and_slow_matrices() {
    assert!(short("Eiffel Tower").starts_with("tower in Paris, France; opened 1889"));
    assert_eq!(short("length Three Gorges Dam / Hoover Dam"), "Three Gorges Dam: 2335 m (7661 ft) | Hoover Dam: 379 m (1243 ft)");
    assert_eq!(short("seating capacity Muny Theater vs Broadway Theater"), "Muny: 11000 seats | Broadway Theatre: 1761 seats");
    assert_eq!(short("Barbican Centre, Boston Opera House opened"), "Barbican Centre: 1982 | Boston Opera House: 1928");
    assert!(short("castles in the UK").starts_with("Windsor Castle"));
    assert!(short("mines over 5000 feet").starts_with("Mponeng Gold Mine"));
    assert_eq!(short("how tall is the eiffel tower"), "330 m (1082.68 ft)");
    // a 3 × 3 matrix with Cardano eigenvalues used to take 20 s
    let t = std::time::Instant::now();
    assert_eq!(short("adjugate {{8, 7, 7}, {6, 9, 2}, {-6, 9, -2}}"), "{{-36, 77, -49}, {0, 26, 26}, {108, -114, 30}}");
    assert!(t.elapsed().as_secs() < 8);
}

#[test]
fn notable_texts() {
    assert!(short("1 Corinthians 13:11").starts_with("When I was a child, I spake as a child"));
    assert_eq!(short("number of words in the US constitution"), "about 4,543");
    assert!(short("books by Sinclair Lewis").contains("Main Street | Babbitt"));
    assert_eq!(short("number of chapters in On the Origin of Species, Walden"), "On the Origin of Species: 14 | Walden: 18");
    assert_eq!(short("number of scenes in the Tempest act 3"), "3");
    assert_eq!(short("compare the number of scenes in King Lear, King John, King Richard II"), "King Lear: 26 | King John: 16 | Richard II: 19");
    assert_eq!(short("When was Dune published?"), "1965");
    assert_eq!(short("original language of War and Peace, Lolita"), "War and Peace: Russian (with substantial passages in French) | Lolita: English");
    assert!(short("Anthony and Cleopatra").starts_with("tragedy (play) by William Shakespeare"));
    assert_eq!(short("who wrote moby dick"), "Herman Melville");
}

#[test]
fn human_anatomy() {
    assert!(short("femur").starts_with("Femur (bone)"));
    assert_eq!(short("origin of biceps brachii"), "short head: coracoid process of the scapula; long head: supraglenoid tubercle of the scapula");
    assert_eq!(short("insertion of soleus"), "posterior surface of the calcaneus via the calcaneal (Achilles) tendon");
    assert_eq!(short("glenohumeral joint articulating bones"), "head of the humerus and the glenoid cavity of the scapula");
    assert_eq!(short("what is the fetus size at Carnegie stage 7"), "stage 7: 0.4 mm");
    assert!(short("venous drainage of large intestine and small intestine").contains("small intestine: superior mesenteric vein"));
    assert!(short("What are the bones of the leg and the foot?").starts_with("femur | patella | tibia | fibula"));
    assert!(short("what are the regional parts of the neural tube").starts_with("prosencephalon"));
    assert!(short("ileum and jejunum").starts_with("Ileum (organ)"));
    assert!(short("how far can a jaguar jump").contains("strongest bite"));
}

#[test]
fn stars_and_stellar_physics() {
    assert!(short("spectral class G2V").starts_with("G2V: a yellow main-sequence star"));
    assert!(short("spectral class A2Ia").contains("luminous supergiant"));
    assert_eq!(short("spectral class Pollux"), "K0III");
    assert_eq!(short("What will a class B9V star turn into when it dies?"), "a class B9V star will end as a carbon–oxygen white dwarf");
    assert!(short("9800K star").contains("spectral class about A0 (peak emission at 295.7 nm)"));
    assert!(short("star 0.65 solar luminosities").contains("0.8979 M☉"));
    assert!(short("10 solar mass supergiant star").ends_with("ends as a neutron star"));
    assert!(short("Sirius mass").starts_with("2.063 M☉"));
    assert_eq!(short("surface temperature Arcturus, Rigel"), "Arcturus: 4286 K | Rigel: 12100 K");
    assert!(short("color Aldebaran").starts_with("orange"));
    assert!(short("variability period of Mira").starts_with("332 days"));
    assert!(short("supernova 1987A").contains("Large Magellanic Cloud"));
    assert!(short("Crab pulsar, Vela pulsar").contains("33.5 ms"));
    assert!(short("HR diagram Deneb, Pollux, Sirius").contains("supergiants"));
    assert!(short("10 nearest stars").contains("Proxima Centauri"));
    assert!(short("Orion").starts_with("Orion: the Hunter"));
    assert_eq!(short("Shapley–Sawyer concentration classification of Omega Centauri"), "VIII");
    assert_eq!(short("average radius of M3"), "about 90 light years");
    assert!(short("How far is NGC 4603 from Earth?").starts_with("108 million"));
    assert!(short("NGC 2808, M 53, NGC 288").contains("NGC 2808 | globular cluster"));
    assert!(!noesis::query("cancer").pods.iter().any(|p| p.subpods.iter().any(|s| noesis::engine::content_text(&s.content).contains("Crab"))));
}

#[test]
fn countries_languages_and_religions() {
    assert!(short("country .dk").contains("Copenhagen"));
    assert!(short("country DJI").contains("Djibouti"));
    assert!(short("Norway, Sweden, Finland").contains("Oslo"));
    assert!(short("countries with area > India and population < Spain").contains("Canada"));
    assert!(short("german speaking country with the largest land area").starts_with("Germany"));
    assert!(short("Arabic speaking countries").contains("Saudi Arabia"));
    assert_eq!(short("native speakers of French"), "about 81 million");
    assert!(short("Romance languages").starts_with("Spanish | Portuguese | French"));
    assert!(short("Basque, Finnish").contains("language isolate"));
    assert_eq!(short("When was Scientology founded?").get(..4), Some("1954"));
    assert_eq!(short("When was Luther's thesis written?"), "31 October 1517");
    assert!(short("the major text of Islam").starts_with("the Quran"));
    assert_eq!(short("Islamic vs Hindu populations in India"), "Muslims: 172.2 million (14.2%) | Hindus: 966.3 million (79.8%)");
    assert_eq!(short("Hindu + Buddhist population UK"), "1.303 million (2.2% of the population)");
    assert!(short("Zoroastrianism").contains("Zoroaster"));
    assert_eq!(short("capital of france"), "Paris");
}

#[test]
fn characters_soundex_and_morse() {
    assert_eq!(short("... .--. .. -.- . -.--"), "SPIKEY");
    assert_eq!(short("morse code ... --- ..."), "SOS");
    assert_eq!(short("Soundex Wolfram"), "W416");
    assert!(short("Soundex code A626").contains("archer"));
    assert!(short("sounds like Rubin").contains("robin"));
    assert_eq!(short("ASCII 32 to 39"), "(space)!\"#$%&'");
    assert_eq!(short("&#42;"), "*");
    assert_eq!(short("u umlaut"), "ü");
    assert_eq!(short("degree symbol"), "°");
    assert!(short("French letters").starts_with("é è ê"));
    assert_eq!(short("letter K"), "K");
    assert!(!noesis::query("...").success);
    assert!(!noesis::query("?").success);
}

#[test]
fn machines_and_inventions() {
    assert_eq!(short("crew of Apollo 13"), "Jim Lovell, Jack Swigert, Fred Haise");
    assert_eq!(short("Vostok 1, Apollo 15, STS 2 launch dates"), "Vostok 1: 12 April 1961 | Apollo 15: 26 July 1971 | STS-2: 12 November 1981");
    assert_eq!(short("How many wheels does the Opportunity rover have?"), "6");
    assert_eq!(short("mass of the Curiosity rover"), "899 kg");
    assert!(short("Voyager 2 velocity").starts_with("15.4 km/s"));
    assert_eq!(short("rockets with mass > 3000 short tons"), "Starship | Saturn V | N1");
    assert!(short("When did the Costa Concordia sink?").starts_with("13 January 2012"));
    assert!(short("How far is the Titanic from New York?").starts_with("2009 km"));
    assert!(short("Sandstone Zebra nuclear explosion height of burst").starts_with("tower, 61 m"));
    assert!(short("largest nuclear explosions").starts_with("Tsar Bomba"));
    assert!(short("who invented the cell phone").starts_with("Martin Cooper"));
    assert_eq!(short("when was the rubik's cube invented"), "1974");
    assert!(short("inventions by Benjamin Franklin").contains("bifocals"));
    assert!(short("How fast was the processor on the Atari 400?").contains("1.79 MHz"));
    assert!(short("earliest programming languages").starts_with("Plankalkül"));
    assert!(short("hiroshima").contains("Japan"));
}

#[test]
fn liquids_materials_spectra_and_black_holes() {
    // liquids, gases and fuels
    assert_eq!(short("surface tension of hexadecane at 25C"), "27.05 mN/m at 25 °C");
    assert!(short("flash point methane, butane, octane").starts_with("Methane (CH₄): -188 °C"));
    assert!(short("viscosity glycerol / methanol").contains("934 mPa·s"));
    assert!(short("tlv of pentane").starts_with("1000 ppm"));
    assert!(short("combustion heat of C3H8").starts_with("ΔcH° = −2219.2 kJ/mol"));
    assert!(short("energy content of H2").contains("141.77 MJ/kg"));
    assert!(short("How fast does sound travel in chloroform?").starts_with("987 m/s"));
    assert!(short("refractive index of natural diamond").starts_with("2.417"));
    assert_eq!(short("speed of sound in water"), "1482 m/s");
    assert!(short("saturation vapor pressure 30C").starts_with("42.42"));
    // polymers, bulk materials, minerals and gems
    assert!(short("35 cubic yards crushed stone").starts_with("42.815 t"));
    assert_eq!(short("chemical formula of nylon"), "(C₁₂H₂₂N₂O₂)ₙ");
    assert_eq!(short("what is the chemical formula of asbestos"), "Mg₃(Si₂O₅)(OH)₄");
    assert_eq!(short("olivine Mohs hardness"), "6.5–7");
    assert!(short("mass of the Delong Star Ruby").starts_with("100.32 carats"));
    assert!(short("asphalt r-value").starts_with("R-0.19"));
    assert!(short("steel").contains("7.87 g/cm³"));
    assert!(short("gasoline").contains("0.745 g/cm³"));
    // street-name synonyms in the compound list are not indexed
    assert!(!short("snow density").contains("1.22"));
    for q in ["coke", "crystal", "abs", "earth"] {
        assert!(!noesis::query(q).pods.iter().any(|p| noesis::engine::content_text(&p.subpods[0].content).contains("C17H21NO4")), "{q}");
    }
    assert!(short("earth").contains("planet"));
    // cosmic origins and emission spectra
    assert!(short("What are the cosmic origins of C & Au?").contains("Gold: merging neutron stars"));
    assert!(short("Ne emission spectrum").contains("640.2"));
    assert!(short("emission spectrum of potassium").contains("766.5"));
    // black holes
    assert!(short("7 nearest black holes").starts_with("Gaia BH1 | Gaia BH3"));
    assert!(short("mass of M87 black hole/M33 black hole").contains("M33 X-7: 15.65 M☉"));
    assert!(short("supermassive black holes").starts_with("TON 618"));
    assert!(short("entropy of V0381 Normae black hole, GU Muscae black hole").contains("k_B | GU Muscae"));
}

#[test]
fn physical_geography() {
    assert_eq!(short("length Amazon / Nile"), "Amazon River: 6400 km (3977 mi) | Nile: 6650 km (4132 mi)");
    assert!(short("Mt Hood").contains("3429 m"));
    assert!(short("Mount Everest").contains("8848.86 m"));
    assert_eq!(short("how tall is mount everest"), "8848.86 m (29,032 ft)");
    assert!(short("surface area Lake Erie + Lake Huron + Lake Ontario").starts_with("104,260 km²"));
    assert_eq!(short("Pacific ocean lowest point"), "Challenger Deep, Mariana Trench");
    assert_eq!(short("volcanic arc of mt rainier volcano"), "Cascade Volcanic Arc");
    assert!(short("most destructive volcanic eruption").starts_with("Mount Tambora"));
    assert!(short("5 highest mountains").starts_with("Mount Everest | K2 | Kangchenjunga"));
    assert!(short("caves in Italy").contains("Blue Grotto"));
    assert!(short("distance from Blue Grotto to Tunis").starts_with("544 km"));
    assert!(short("Tigris, Euphrates").contains("Euphrates"));
    assert!(short("width Victoria Falls / Iguacu Falls").contains("Iguazu Falls: 2700 m"));
    // country and city names are not taken
    assert!(short("kenya").contains("Nairobi"));
    assert!(short("jordan").contains("Amman"));
}

#[test]
fn molecular_biology_and_dinosaurs() {
    assert_eq!(short("reverse complement of ATGC"), "5′-GCAT-3′");
    assert_eq!(short("AAGCTAGCTAGC"), "12-nucleotide DNA sequence, 50% GC");
    assert_eq!(short("melting point GCTCGTAGCTACGT"), "40.3 °C");
    assert!(short("ATTGGGCAAAAA molecular weight").starts_with("3702.51 g/mol"));
    assert_eq!(short("translate AUGGCCUAA"), "MA*");
    assert_eq!(short("alanine codons"), "GCU, GCC, GCA, GCG");
    assert_eq!(short("stop codons"), "UAA, UAG, UGA");
    assert!(short("codon AUG").starts_with("methionine"));
    assert_eq!(short("how many chromosomes orangutan vs human vs chimp"), "orangutan: 48 | human: 46 | chimpanzee: 48");
    assert!(short("how many chromosomes does a dog have").starts_with("78"));
    assert!(short("insulin amino acid sequence").contains("GIVEQCCTSICSLYQLENYCN"));
    assert!(short("TCA cycle").contains("acetyl-CoA"));
    assert!(short("homologs of CCR5 gene").contains("mouse Ccr5"));
    assert!(short("E. coli mutS and human MSH2 genes").starts_with("E. coli mutS"));
    assert!(short("triceratops").contains("Late Cretaceous"));
    assert!(short("saurischia").contains("lizard-hipped"));
    assert!(short("largest dinosaur").starts_with("Argentinosaurus"));
    assert!(short("mass of a cat") != "4-nucleotide DNA sequence, 50% GC");
}

#[test]
fn famous_people() {
    assert!(short("Why was Albert Einstein famous?").contains("relativity"));
    assert!(short("Charlemagne").contains("(c. 747–814)"));
    assert_eq!(short("Mary Shelley's place of birth"), "Somers Town, London, England");
    assert_eq!(short("when was Galileo born"), "Saturday, February 15, 1564");
    assert_eq!(short("when did Elvis die"), "Tuesday, August 16, 1977");
    assert!(short("Betty White age").starts_with("99 years"));
    assert!(short("famous people from Detroit").contains("Diana Ross"));
    assert!(short("famous people named Stephen").contains("Stephen Hawking"));
    assert!(short("Nobel Prize winners from France").contains("Marie Curie"));
    assert!(short("Martin Luther King, Jr., Mother Teresa, Mahatma Gandhi").contains("Mother Teresa"));
    assert!(short("Robert Pattinson birthday moon phase").contains("illuminated"));
    assert!(short("Paris Hilton").contains("media personality"));
    // surnames that are also cities or units stay with geography and units
    assert!(!short("darwin").contains("naturalist"));
}

#[test]
fn history_eras_and_battles() {
    assert_eq!(short("How long did the Aztec empire last?"), "about 93 years");
    assert_eq!(short("how long did the roman empire last"), "about 502 years");
    assert_eq!(short("roman republic demonym"), "Roman");
    assert_eq!(short("nationality name for the kingdom of burgundy"), "Burgundian");
    assert!(short("Which current countries were ruled by the Mongolian empire?").starts_with("Mongolia, China, Russia"));
    assert!(short("Ottoman Empire in 1566").contains("Suleiman"));
    assert_eq!(short("time between the start of the Battle of Kadesh and the end of the Battle of Actium"), "1,243 years");
    assert!(short("commanders during the Battle of Pharsalus").starts_with("Julius Caesar"));
    assert!(short("US civil war battles").contains("Battle of Gettysburg"));
    assert!(short("Bear Flag Republic").contains("June 14, 1846"));
    assert!(short("Ancient Athens vs Sparta").contains("Sparta"));
    assert!(short("When was the great Baltimore fire?").contains("February 7, 1904"));
    // Wikidata years are astronomical: Actium was 31 BC
    assert!(short("when was the battle of actium").contains("31 BC"));
}

#[test]
fn sports_teams_venues_and_family() {
    assert!(short("mass of a bowling ball vs. cricket ball").contains("cricket ball: 155.9 g"));
    assert_eq!(short("head width of tennis racket"), "31.8 cm (12.5 in)");
    assert!(short("Dallas Cowboys").contains("Arlington, Texas"));
    assert!(short("Fenway Park").contains("April 20, 1912"));
    assert_eq!(short("How much did it cost to build Metlife stadium?"), "$1.6 billion");
    assert_eq!(short("How many medals has Michael Phelps won?"), "28 Olympic medals (23 gold, 3 silver, 2 bronze)");
    assert_eq!(short("Prince William's parents"), "King Charles III, Diana, Princess of Wales");
    assert!(short("who was Liz Taylor married to?").contains("Richard Burton"));
    assert!(short("Karl Malone vs. Bill Russell").contains("Bill Russell"));
    // a bare plural animal is not a team
    assert!(!short("dolphins").contains("NFL"));
}

#[test]
fn travel_teeth_and_brain_areas() {
    assert_eq!(short("distance from Rome to Naples in kilometers"), "186.7 km");
    assert_eq!(short("travel time from Tokyo to Sydney by aircraft"), "10 h 10 min");
    assert!(short("time to travel from Hong Kong to Honolulu by ship").contains("days"));
    assert_eq!(short("tooth #5"), "upper right first premolar");
    assert_eq!(short("Brodmann area 4"), "primary motor cortex");
    assert!(short("nerve supply of gallbladder").contains("vagus"));
}

#[test]
fn nutrition_meals_and_daily_values() {
    assert_eq!(short("calories in a whole banana"), "105 kcal");
    assert!(short("2 slices of swiss cheese").starts_with("213 kcal"));
    assert_eq!(short("sodium in white bread vs wheat bread"), "1 slice white bread: 123 mg; 1 slice whole wheat bread: 144 mg");
    assert_eq!(short("calories in 1 bowl of corn flakes + 1/2 cup of lowfat milk + a glass of OJ"), "263 kcal");
    assert_eq!(short("Vitamin A in 250g sweet potato"), "1773 µg");
    assert_eq!(short("iron DRI for 65 year old male"), "8 mg per day");
    assert_eq!(short("fiber DRI for 30 year old female"), "25 g per day");
    assert_eq!(short("daily value for vitamin C / vitamin c in an apple"), "10.8");
    assert!(short("caffeine in 24 oz. coffee, 24 oz. soda").contains("284 mg"));
    assert!(short("what size turkey is needed to feed 10 adults and 5 children").starts_with("13–19 lb"));
    // a bare ambiguous word is not a food
    assert!(!short("turkey").contains("kcal"));
    assert!(!noesis::query("apple").short_answer().unwrap_or_default().contains("kcal"));
}

#[test]
fn combined_lookups() {
    assert_eq!(short("population Madrid / population Spain"), "0.07319 (7.319%)");
    // the right side borrows the property from the left
    assert_eq!(short("GDP per capita Norway / United States"), "1.155");
    assert!(short("population Madrid").contains("3,506,730"));
    // ordinary maths is untouched
    assert_eq!(short("sin x / cos x"), "tan(x)");
}

#[test]
fn fixes_from_wolframalpha_comparison() {
    assert_eq!(short("factor sin x + sin y"), "2 cos((x - y)/2) sin((x + y)/2)");
    assert!(short("anagrams listen").contains("silent"));
    assert!(short("luminosity 10 solar masses").starts_with("4427 L☉"));
    assert!(short("GF(8)").starts_with(" | 0 | 1 | a | b"));
    assert_eq!(short("GRS80"), "Geodetic Reference System of 1980");
    assert_eq!(short("GDA94"), "Geodetic Reference System of 1980");
}

#[test]
fn country_statistics() {
    assert_eq!(short("life expectancy in New Zealand"), "82.9 years (2024 est.)");
    assert!(short("life expectancy U.S., Sweden, Japan").starts_with("United States: 80.9 years"));
    assert!(short("flag of Sri Lanka").starts_with("yellow with two panels"));
    assert!(short("unemployment rate Spain vs Italy").contains("Italy: 6.8%"));
    assert!(short("countries with the highest inflation").starts_with("Venezuela"));
    assert_eq!(short("Germany electricity consumption / population"), "6146 kWh per person");
    // GDP per capita stays nominal, from the existing data
    assert_eq!(short("GDP per capita Norway / United States"), "1.155");
}

#[test]
fn airports() {
    assert_eq!(short("LAX"), "Los Angeles International Airport (LAX) — Los Angeles, CA, United States");
    assert!(short("JFK, LGA, EWR").contains("LaGuardia Airport (LGA)"));
    assert!(pod_text(&q("JFK, LGA, EWR"), "Distances").contains("JFK – LGA"));
    assert_eq!(short("ICAO code of Heathrow airport"), "EGLL");
    assert_eq!(short("elevation of Denver airport"), "5431 ft (1655 m)");
    assert_eq!(short("distance from JFK to LAX"), "3974 km (2469 miles, 2146 nmi)");
    assert_eq!(short("flight time from LAX to NYC"), "5 h 23 min");
    assert!(short("airports near Chicago").contains("(ORD)"));
    // codes that read first as something else are not taken as airports
    assert!(!short("CAT").contains("Airport"));
    assert!(noesis::query("SIN").short_answer().unwrap_or_default().find("Airport").is_none());
}

#[test]
fn money_defaults_banknotes_and_clouds() {
    // currency arithmetic (offline, approximate rates)
    assert!(short("150 USD + 80 EUR").ends_with(" USD"));
    assert!(short("2500 euros").ends_with(" USD"));
    assert!(short("1300 Mexican pesos / 20,000 Russian rubles").parse::<f64>().is_ok());
    assert!(short("5 pounds").contains("kg"));
    // loans with typical terms assumed
    assert_eq!(short("mortgage $300,000 30 years 6%"), "$1,798.65");
    assert_eq!(short("car loan $20,000"), "$396.02");
    assert!(pod_text(&q("mortgage"), "Assumptions").contains("30-year fixed mortgage"));
    assert_eq!(short("time to pay $1200 credit card"), "25 months (2.1 years)");
    assert!(short("student loan 30000").starts_with('$'));
    // US banknotes
    assert_eq!(short("width of a dollar bill"), "2.61 in (66.3 mm)");
    assert_eq!(short("who is on the $50 bill"), "Ulysses S. Grant");
    // clouds
    assert!(short("nimbostratus, cumulus").starts_with("nimbostratus: middle"));
    assert!(short("cumulus cloud height").starts_with("base 0.5–2 km"));
}

#[test]
fn moons_and_minor_bodies() {
    assert!(pod_text(&q("Titan"), "Properties").contains("distance from Saturn | 1,221,870 km"));
    assert!(short("Io, Europa, Ganymede, Callisto").contains("Callisto | moon of Jupiter"));
    assert!(short("Ceres, Pallas, Juno, Vesta").contains("Vesta | asteroid"));
    assert!(pod_text(&q("Triton moon"), "Properties").contains("(retrograde)"));
}

#[test]
fn measuring_instruments() {
    assert_eq!(short("what does a ruler measure?"), "length");
    assert_eq!(short("alternative names for an hourglass"), "sandglass | sand timer | sand clock | egg timer");
    assert!(short("device used to measure time").contains("hourglass"));
    assert!(short("instances of a particle detector").contains("bubble chamber"));
    assert!(short("thermometer types").starts_with("mercury thermometer"));
    assert!(short("hydrometer").starts_with("Hydrometer — measures density"));
    assert!(short("how do you measure charges?").contains("electrometer"));
}

#[test]
fn pronunciation_and_rhymes() {
    assert_eq!(short("pronunciation of colonel"), "/ˈkɝnəl/");
    assert_eq!(short("phonetic hypothesize"), "/haɪˈpɑθəˌsaɪz/");
    assert_eq!(short("syllables in banana"), "3");
    assert!(short("rhymes with house").contains("mouse"));
    assert!(short("words close to house").contains("horse"));
    assert!(short("word plinth").contains("architectural support"));
    assert!(pod_text(&q("define plinth"), "Pronunciation").contains("/plɪnθ/"));
}

#[test]
fn broader_and_narrower_terms() {
    assert!(short("broader terms wine").starts_with("alcohol"));
    assert!(short("narrower terms green").contains("sea green"));
    assert!(pod_text(&q("define dog"), "Broader").contains("canine"));
}

#[test]
fn continents_compass_and_aggregates() {
    assert_eq!(short("area of Africa"), "30,370,000 km² (11,725,923 mi²)");
    assert!(short("highest elevation of South America vs North America").starts_with("South America: Aconcagua"));
    assert_eq!(short("SSW"), "202.5° (range 191.25° to 213.75°)");
    assert!(short("world gdp per capita").ends_with("per person (2024 est.)"));
    // acronyms are not products of variables
    assert!(!noesis::query("EU gdp").short_answer().unwrap_or_default().contains("E U"));
    // a hyphenated name is not a graph edge
    assert_eq!(short("carbon-14 half-life"), "5700 years");
    // fractions, mixed numbers and multiples in conversions
    assert_eq!(short("1 1/3 quarts times two in fluid ounces"), "85.333333 US fluid ounces");
    assert_eq!(short("mass Proxima Centauri / Sun"), "0.1222 (12.22%)");
}

#[test]
fn layers_and_buildings() {
    assert_eq!(short("asthenosphere"), "100–350 km below the surface");
    assert!(short("50,000km beneath the surface of the sun").starts_with("convection zone"));
    assert!(short("radiative zone of the sun").starts_with("energy moves outward as radiation"));
    assert!(short("Empire State Building, Chrysler Building").contains("Chrysler Building: height 318.9 m"));
    assert_eq!(short("UHF"), "300 MHz–3 GHz");
}

#[test]
fn word_problems_cousins_and_us_cities() {
    assert_eq!(short("Jordan has 6 basketballs and Shaquille has 4. How many do they have together?"), "10 basketballs");
    assert_eq!(short("Tom has 3 apples and buys 5 more. He eats 2. How many apples does Tom have now?"), "6 apples");
    assert_eq!(short("Ann has 12 pencils and Bob has 7. How many more pencils does Ann have than Bob?"), "5 pencils");
    assert_eq!(short("Ann has 10 stickers and Bob has 2. Ann gives 3 to Bob. How many stickers does Bob have?"), "5 stickers");
    assert!(short("second cousin once removed").starts_with("the child of your second cousin"));
    // a state after the comma picks the city in that state
    assert!(short("Springfield, MA").contains("42.10° N, 72.59° W"));
    assert!(short("Springfield, Illinois").contains("39.80° N"));
}

#[test]
fn special_values_and_bare_matrices() {
    assert_eq!(short("erf(3)"), "0.999977909503");
    assert_eq!(short("Beta(5, 4)"), "1/280");
    assert_eq!(short("{{6,-7},{0,3}}"), "18");
    assert!(short("OEIS A000217").starts_with("1, 3, 6, 10"));
    assert_eq!(short("230 grams/2 in ounces"), "4.0565056 ounces");
}

#[test]
fn yoga_pilates_and_strength_exercises() {
    assert!(short("Pincha Mayurasana").starts_with("yoga inversion (advanced)"));
    assert!(short("hundred pilates primary contracted muscles").starts_with("abdominal"));
    assert!(short("advanced Pilates exercises").contains("Teaser"));
    assert!(short("isolated weight training exercises").contains("Biceps Curl"));
    assert!(short("how do you do a squat").contains("thighs are about parallel"));
    assert!(short("Sun Salutation A").starts_with("Mountain Pose (Tadasana)"));
    assert!(short("yoga poses to strengthen abdominal muscles").starts_with("Plank Pose"));
}

#[test]
fn us_states() {
    assert_eq!(short("capital of Vermont"), "Montpelier");
    assert_eq!(short("Texas state flower"), "bluebonnet");
    assert_eq!(short("New Mexico's state bird"), "greater roadrunner");
    assert_eq!(short("when did Hawaii become a state"), "August 21, 1959 (50th state)");
    assert!(short("illinois income tax vs. virginia income tax vs. oregon income tax").starts_with("Illinois: 4.95%"));
    assert!(short("largest states by area").starts_with("Alaska"));
    // the country, a city and a chemical keep their meanings
    assert_eq!(short("capital of Georgia"), "Tbilisi");
    assert!(!noesis::query("CO").short_answer().unwrap_or_default().contains("Colorado"));
}

#[test]
fn bare_titles_and_works_by_creator() {
    assert_eq!(short("Girl with a Pearl Earring"), "painting by Johannes Vermeer");
    assert!(short("Rear Window vs Vertigo").starts_with("Rear Window: film by Alfred Hitchcock (1954)"));
    assert!(short("Sasuke Uchiha").contains("Naruto"));
    assert!(short("paintings by Frida Kahlo").contains("The Two Fridas"));
    // ordinary meanings come first
    assert!(short("Mars").contains("planet"));
}

#[test]
fn file_formats_and_keyboard_layouts() {
    assert_eq!(short(".pdf file format"), "Portable Document Format (document file)");
    assert_eq!(short("mime type of mp3"), "audio/mpeg");
    assert!(short("dvorak keyboard").contains("AOEUIDHTNS"));
    assert!(short("english keyboard layout").starts_with("US QWERTY"));
}

#[test]
fn periodic_table_radioactive_elements_and_uk_nations() {
    assert!(short("radioactive elements").starts_with("Technetium, Promethium, Polonium"));
    assert!(pod_text(&q("show me the periodic table"), "Result").contains("Og"));
    assert_eq!(short("capital of Northern Ireland"), "Belfast");
    assert!(short("Scotland").contains("capital Edinburgh"));
}

#[test]
fn tool_examples_and_named_cubic_graphs() {
    assert_eq!(short("linear fit"), "2.08 x - 0.15");
    assert!(pod_text(&q("Z-transform calculator"), "Example").contains("Z transform of 2^n"));
    let pappus = pod_text(&q("Pappus graph"), "Properties");
    assert!(pappus.contains("girth | 6") && pappus.contains("diameter | 4"));
}

#[test]
fn log_plots_and_conway_constant() {
    assert!(pod_text(&q("log plot e^x-x"), "Scale").contains("log₁₀"));
    assert!(short("Conway's constant").starts_with("1.30357726903429639125"));
}

#[test]
fn complexity_classes_crystals_and_earth_layers() {
    assert!(short("Is BPL in QP?").starts_with("Yes: BPL ⊆ QP"));
    assert!(short("NC vs NLINSPACE").starts_with("NC ⊊ NLINSPACE"));
    assert!(short("P vs NP").contains("open problem"));
    assert!(short("non-deterministic polynomial time").contains("certificates"));
    assert_eq!(short("ditetragonal pyramidal point group"), "Hermann–Mauguin 4mm | Schoenflies C4v");
    assert!(short("trigonal crystal system").contains("α = β = γ ≠ 90°"));
    assert!(short("crystal structure of iron").starts_with("body-centred cubic"));
    assert!(pod_text(&q("hematite, magnetite, chromite"), "Result").contains("FeCr₂O₄"));
    assert_eq!(short("crust, mantle"), "crust: 0–35 km | mantle: 35–2,891 km");
    assert!(short("atmosphere of Earth").starts_with("nitrogen 78.08%"));
    assert_eq!(short("SHA \"Wolfram Alpha\""), "a362a5fb22cb6fa4c981811457149e11fc3e5f3b");
    assert_eq!(short("Morse code hello"), ".... . .-.. .-.. ---");
    assert_eq!(short("morse code ... --- ..."), "SOS");
}

#[test]
fn seismology_photography_acronyms_and_words() {
    assert!(short("Richter scale 6").starts_with("strong earthquake"));
    assert!(pod_text(&q("Mercalli scale"), "Levels").contains("VIII"));
    assert!(short("define body wave magnitude").starts_with("m_b = log₁₀(A/T)"));
    assert_eq!(short("angle of view 50mm"), "46.793°");
    assert_eq!(short("exposure value f/8 1/125"), "12.9658");
    assert!(pod_text(&q("depth of field f=1m"), "Assumptions").contains("f-number 8"));
    assert_eq!(short("acronym USSS"), "United States Secret Service");
    assert_eq!(short("abbreviation NATO"), "North Atlantic Treaty Organization");
    assert!(short("word subsets recycle").contains("cycle"));
    assert!(short("letter bank tale").contains("latte"));
    assert_eq!(short("primary incisors"), "8 (of 20 primary teeth)");
    assert!(short("OEIS A000219").starts_with("1, 1, 3, 6, 13, 24, 48, 86, 160"));
    assert!(short("recurrences Laguerre polynomials").starts_with("(n + 1) Lₙ₊₁(x)"));
    assert!(short("Petersen graph, icosahedral graph").contains("icosahedral graph: 12 vertices, 30 edges"));
    // "ev" and "dof" are whole words only
    assert!(!noesis::query("events from November 1, 1989 to November 30, 1989").short_answer().unwrap_or_default().contains("12.9658"));
}

#[test]
fn colours_text_length_holidays_and_prayer_times() {
    assert!(short("RGB 0.95, 0.75, 0.00").starts_with("#F2BF00"));
    assert_eq!(short("violet vs magenta"), "violet #EE82EE | magenta #FF00FF");
    assert!(short("dark orange color").starts_with("#FF8C00"));
    assert!(short("70,000 words").starts_with("140 single-spaced pages"));
    assert_eq!(short("120 page screenplay"), "about 2 h 0 min");
    assert!(short("1024 x 768 pixels").contains("XGA"));
    assert!(short("39 desktop publishing points").starts_with("1.37583"));
    assert!(pod_text(&q("Christian holidays 2027"), "Result").contains("Thursday, May 6, 2027"));
    assert_eq!(short("advent 2026").split("  ").next().unwrap(), "Sunday, November 29, 2026");
    assert!(pod_text(&q("prayer times in Cairo"), "Table").contains("Maghrib"));
    assert!(short("population of Cologne").starts_with("1,024,621"));
}

#[test]
fn board_games_ropes_comets_and_lab_ranges() {
    assert_eq!(short("Agricola"), "1–5 players | ages 12+ | 30–150 minutes");
    assert!(short("Axis & Allies vs. Risk").contains("Risk: 2–6 players"));
    assert_eq!(short("how many players are needed for Risk?"), "2–6 players");
    // a bare common word is not a game
    assert!(!noesis::query("Risk").success || !noesis::query("Risk").short_answer().unwrap_or_default().contains("players"));
    assert!(short("21mm nylon rope").starts_with("breaking strength about 83.8 kN"));
    assert!(short("specific gravity of 28mm polyester rope").starts_with("1.38"));
    assert_eq!(short("last perihelion of Comet Holmes"), "February 19, 2021");
    assert_eq!(short("when will halley's comet return"), "July 28, 2061");
    assert!(short("Leonids").contains("November 17"));
    assert_eq!(short("serum blood urea nitrogen"), "7–20 mg/dL");
    assert!(short("creatinine=0.9mg/dL, adult male").starts_with("within the reference range"));
    assert!(short("potassium 5.8 mmol/L").starts_with("above"));
    assert_eq!(short("iodine-123 biological half-life"), "138 days (thyroid)");
    assert_eq!(short("effective half-life of cesium-137"), "69.56 days");
    // calcium-40 stays a nuclide
    assert!(!short("calcium 40").contains("reference range"));
}

#[test]
fn instrument_ranges_and_woods() {
    assert_eq!(short("clarinet"), "D3 to B♭6 | 147 Hz to 1865 Hz");
    assert_eq!(short("What frequencies can a violin sound?"), "G3 to A7 | 196 Hz to 3520 Hz");
    assert_eq!(short("lowest note on a double bass"), "E1 (41.2 Hz)");
    assert!(!noesis::query("tenor").success);
    assert_eq!(short("Young’s modulus of post oak"), "10.4 GPa");
    assert!(short("wood indentation hardness hemlock, red oak, fir").contains("Northern red oak: 1290 lbf"));
    assert!(short("What are the longest board games?").starts_with("Risk (1–8 hours)"));
    assert!(short("what is air made of?").starts_with("nitrogen 78.08%"));
}

#[test]
fn ac_voltage_tires_and_cryptograms() {
    assert_eq!(short("110V AC"), "110 V RMS | peak 155.56 V | peak-to-peak 311.13 V");
    assert_eq!(short("peak voltage of 120 V AC"), "169.71 V");
    // a plain voltage is still a quantity
    assert_eq!(short("110 V"), "110 V");
    assert!(short("bicycle tire sizes").starts_with("700C / 29 in (ISO 622)"));
    assert_eq!(short("imperial size 15 knitting needle"), "1.75 mm");
    assert!(short("Mali declares independence").starts_with("22 September 1960"));
    assert!(short("cryptograms calculus").contains("volvulus"));
}

#[test]
fn reaction_products_and_compound_classes() {
    assert_eq!(short("hydrogen + oxygen ->"), "2 H₂ + O₂ → 2 H₂O");
    assert_eq!(short("sodium reacts with water"), "2 Na + 2 H₂O → 2 NaOH + H₂");
    // two elements joined by "and" are still compared
    assert!(short("hydrogen and oxygen").contains("atomic number"));
    assert!(short("mineral acids").starts_with("hydrochloric acid | nitric acid"));
    assert!(short("nucleosides").starts_with("adenosine | guanosine"));
    assert!(pod_text(&q("amino acids"), "Table").contains("C₁₁H₁₂N₂O₂"));
}

#[test]
fn dog_and_horse_anatomy_and_bare_activities() {
    assert!(short("dog femur").starts_with("long bone of the thigh"));
    assert_eq!(short("horse short pastern homologous human structures"), "middle phalanx of the middle finger or toe");
    assert!(short("what does the dog tibia connect to").starts_with("the femur at the stifle joint"));
    assert!(short("how many bones are in the dog's left hindfoot").starts_with("about 23"));
    assert!(pod_text(&q("horse left stifle joint image view from left"), "Table").contains("knee"));
    assert_eq!(short("unicycling"), "350 kcal");
}

#[test]
fn integral_tables_and_named_constants() {
    assert!(short("integrals containing cos(u)").starts_with("∫cos(u) du = sin(u) + C"));
    assert!(short("definite integrals with Si(x)").contains("Si(1) + cos(1) − 1"));
    assert!(short("sample definite integrals product log").starts_with("∫₀^e W(x) dx = e − 1"));
    // a single integral is still computed
    assert!(short("integral of cos(x)").contains("sin(x)"));
    assert_eq!(short("log 2 log (Khinchin's constant)"), "0.684725");
    assert_eq!(short("Catalan's constant * 2"), "1.83193");
}

#[test]
fn geodetic_lists_and_solvent_miscibility() {
    assert_eq!(short("Airy 1830, Modified Fischer 1960"), "Airy 1830 ellipsoid: 6377563.396 m | Modified Fischer 1960 ellipsoid: 6378155 m");
    assert!(short("NAD27, ITRF00").contains("International Terrestrial Reference Frame 2000"));
    assert_eq!(short("is ethanol soluble in benzene?"), "yes: miscible in all proportions");
    assert_eq!(short("is hexane miscible with methanol"), "no: immiscible (two layers)");
    assert!(short("What solvents are miscible in acetone?").starts_with("water | methanol"));
}

#[test]
fn element_conductivity_and_magnetism() {
    assert_eq!(short("conductivity sulfur, silicon, copper"), "Sulfur: 1×10^-15 S/m | Silicon: 4.35×10^-4 S/m | Copper: 5.95×10^7 S/m");
    assert!(short("elements sorted by conductivity").starts_with("Silver (6.29×10^7 S/m) | Copper"));
    assert_eq!(short("iron magnetism"), "ferromagnetic");
    assert!(short("is gold magnetic").contains("diamagnetic"));
    assert_eq!(short("resistivity of copper"), "1.68×10^-8 Ω m");
    assert_eq!(short("thermal conductivity of copper"), "401 W/(m K)");
}

#[test]
fn artworks_names_and_germination() {
    assert_eq!(short("The Kiss by Rodin"), "Auguste Rodin | 1882 (marble carved 1888–1898) | Musée Rodin, Paris, France");
    assert_eq!(short("which museum exhibits Pieta by Michelangelo?"), "St. Peter's Basilica, Vatican City");
    assert_eq!(short("how big is the mona lisa"), "77 cm × 53 cm");
    assert!(short("name William").starts_with("resolute protector"));
    assert!(short("What does the name Cassandra mean?").contains("shining upon man"));
    assert!(short("name Jones").starts_with("son of John"));
    assert_eq!(short("5 most popular girl's names"), "Olivia | Emma | Amelia | Charlotte | Mia");
    assert_eq!(short("germination time for garden peas"), "7–14 days (at 75 °F soil)");
    assert_eq!(short("days to maturity for sweet corn"), "60–100 days from seed");
}

#[test]
fn character_creators_and_comparisons() {
    assert_eq!(short("Who created Spiderman?"), "Stan Lee  |  Steve Ditko");
    assert_eq!(short("who created superman"), "Jerry Siegel  |  Joe Shuster");
    assert_eq!(short("who created Bugs Bunny and Daffy Duck?"), "Bugs Bunny: Tex Avery | Daffy Duck: Bob Clampett, Tex Avery");
    assert!(short("Captain America vs. Iron Man").starts_with("Captain America: superhero"));
    assert!(short("Edward Cullen, Isabella Cullen").contains("Bella Swan"));
    assert_eq!(short("who created the Mona Lisa"), "Leonardo da Vinci");
}

#[test]
fn arithmetic_on_looked_up_quantities() {
    assert_eq!(short("(mass Phobos+Deimos) / (mass Mars)"), "1.89107×10^-8");
    assert_eq!(short("(mass Saturn+Uranus+Neptune) / (mass Jupiter)"), "0.399098");
    assert_eq!(short("(surface area Ceres) / (area India)"), "0.843283");
}

#[test]
fn orbits_reagents_and_packing() {
    assert_eq!(short("What is the semi major axis of an orbit with an apogee of 2000km and perigee of 275km?"), "1137.5 km");
    assert_eq!(short("orbital radius with eccentricity 0.7 and semimajor axis 17.8 AU"), "5.34 AU to 30.26 AU");
    assert_eq!(short("concentrated acetic acid"), "17.4 mol/L");
    assert_eq!(short("concentrated sulfuric acid normality"), "36.8 N");
    assert_eq!(short("How many marbles fit in 1000 cubic inches?"), "about 4,890");
    assert_eq!(short("how many golf balls fit in a gallon"), "about 60");
}

#[test]
fn nobel_years_verses_and_mains() {
    assert!(short("Nobel Prize 1930").starts_with("Physics: Chandrasekhara Venkata Raman | Chemistry: Hans Fischer"));
    assert_eq!(short("Gospel of John chapter 11 verse 35"), "Jesus wept.");
    assert_eq!(short("mains voltage in Bhutan"), "230 V, 50 Hz");
    assert_eq!(short("plug type Japan"), "types A, B");
    assert_eq!(short("110V AC"), "110 V RMS | peak 155.56 V | peak-to-peak 311.13 V");
}

#[test]
fn equinoxes_solstices_and_lagrange_points() {
    assert_eq!(short("september equinox 2025"), "Monday, September 22, 2025, 18:19 UTC");
    assert_eq!(short("march equinox 2024"), "Wednesday, March 20, 2024, 03:06 UTC");
    assert_eq!(short("summer solstice 2026"), "Sunday, June 21, 2026, 08:24 UTC");
    assert!(short("autumnal equinox 1850").starts_with("Monday, September 23, 1850"));
    assert!(short("Lagrange points Earth and Moon").starts_with("L1: 58,020 km from Moon | L2: 64,510 km from Moon"));
    assert_eq!(short("Sun-Earth L2"), "1.51 million km from Earth (away from Sun)");
}

#[test]
fn planet_positions_moon_periods_and_synonyms() {
    assert!(short("sun position on 2025-06-21").starts_with("RA 5h 58m"));
    assert!(short("sun position on 2025-06-21").contains("Dec +23° 26′"));
    assert!(short("next waning gibbous moon").starts_with("waning gibbous: "));
    assert!(short("synonyms granular").contains("grainy"));
}

#[test]
fn carbon_footprints_dated_positions_and_can_volumes() {
    assert_eq!(short("carbon footprint coal 100W"), "104.3 g per hour | 914.29 kg per year");
    assert_eq!(short("carbon footprint driving 536 miles at 32mpg"), "148.86 kg");
    assert!(short("carbon footprint natural gas").starts_with("0.44 kg CO₂ per kWh of electricity"));
    assert!(short("mars on 2020-10-13").ends_with("in Pisces"));
    assert_eq!(short("2 cans of coke in ml."), "710 ml");
}

#[test]
fn bare_offices_and_opening_lines() {
    assert!(short("President of Argentina").starts_with("Javier Milei"));
    assert_eq!(short("first line of moby dick"), "Call me Ishmael.");
    assert!(short("opening phrase of A Tale of Two Cities").starts_with("It was the best of times"));
}

#[test]
fn office_holders_at_events() {
    assert!(short("Who was King of England when WWI ended?").starts_with("George V"));
    assert!(short("Prime Minister of Pakistan when Osama Bin Laden was killed?").starts_with("Yusuf Raza Gilani"));
    assert!(short("President of the US 87 years after the Gettysburg Address").starts_with("Harry S. Truman"));
    assert!(short("Leader of Sudan on South Sudan's independence date").starts_with("Omar al-Bashir"));
}

#[test]
fn numeric_distributions_and_parametric_surfaces() {
    assert!(short("moyal distribution mu=2.3 sigma=12 skewness").contains("1.5351"));
    let r = q("3d parametric plot (cos u, sin u + cos v, sin v), u=0 to 2pi, v=0 to 2pi");
    let p = r.pods.iter().find(|p| p.id == "Plot").unwrap();
    match &p.subpods[0].content {
        noesis::engine::Content::Plot { plot: pl } => assert!(pl.curve3d.len() >= 40),
        _ => panic!("no plot"),
    }
}
