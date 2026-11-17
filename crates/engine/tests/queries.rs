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
    assert_eq!(short("Rachel has 17 apples. She gives 9 to Sarah. How many apples does Rachel have now?"), "8");
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
