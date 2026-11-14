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
    let text = concat!(include_str!("data/corpus.txt"), "\n", include_str!("data/corpus2.txt"), "\n", include_str!("data/corpus3.txt"));
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
    assert!(short("BMI 70 kg 175 cm").starts_with("22.9"));
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
    assert!(short("integrate x^2 y dx dy x=0..1 y=0..2").ends_with("= 2/3"));
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
    assert_eq!(short("oxidation states of manganese"), "+2, +4, +7");
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
