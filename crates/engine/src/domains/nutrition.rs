//! Nutrition: foods with amounts ("2 slices of swiss cheese", "250 g sweet
//! potato"), meals ("turkey + provolone cheese + lettuce"), comparisons
//! ("sodium in white bread vs wheat bread"), single nutrients ("iron in
//! 1 cup orange juice"), FDA daily values and DRIs by age and sex.

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (`~` = not accepted as a bare one-word query) | serving units
/// "name:grams;…" (the first is the default serving) | then per 100 g:
/// kcal | protein g | fat g | saturated fat g | cholesterol mg | sodium mg |
/// carbohydrate g | fibre g | sugars g | calcium mg | iron mg | potassium mg |
/// vitamin C mg | vitamin A µg RAE | caffeine mg.
/// Approximate values after USDA FoodData Central (SR Legacy).
const FOODS: &str = "\
banana,bananas|medium:118;large:136;small:101;cup:150|89|1.09|0.33|0.112|0|1|22.84|2.6|12.23|5|0.26|358|8.7|3|0
~apple,apples|medium:182;large:223;small:149;cup:109|52|0.26|0.17|0.028|0|1|13.81|2.4|10.39|6|0.12|107|4.6|3|0
~orange,oranges|medium:131;large:184;small:96;cup:180|47|0.94|0.12|0.015|0|0|11.75|2.4|9.35|40|0.1|181|53.2|11|0
pear,pears|medium:178;large:230;small:148;cup:140|57|0.36|0.14|0.022|0|1|15.23|3.1|9.75|9|0.18|116|4.3|1|0
pineapple,pineapples|cup:165;slice:84;whole:905|50|0.54|0.12|0.009|0|1|13.12|1.4|9.85|13|0.29|109|47.8|3|0
grapes,red grapes,green grapes,grape|cup:151;grape:5|69|0.72|0.16|0.054|0|2|18.1|0.9|15.48|10|0.36|191|3.2|3|0
strawberries,strawberry|cup:152;medium:12;large:18|32|0.67|0.3|0.015|0|1|7.68|2|4.89|16|0.41|153|58.8|1|0
blueberries,blueberry|cup:148|57|0.74|0.33|0.028|0|1|14.49|2.4|9.96|6|0.28|77|9.7|3|0
watermelon|wedge:286;cup:152|30|0.61|0.15|0.016|0|1|7.55|0.4|6.2|7|0.24|112|8.1|28|0
mango,mangos,mangoes|whole:336;cup:165|60|0.82|0.38|0.092|0|1|14.98|1.6|13.66|11|0.16|168|36.4|54|0
~peach,peaches|medium:150;large:175;small:130|39|0.91|0.25|0.019|0|0|9.54|1.5|8.39|6|0.25|190|6.6|16|0
~lemon,lemons|medium:58|29|1.1|0.3|0.039|0|2|9.32|2.8|2.5|26|0.6|138|53|1|0
~cherries,cherry,sweet cherries|cup:154;cherry:8.2|63|1.06|0.2|0.038|0|0|16.01|2.1|12.82|13|0.36|222|7|3|0
kiwifruit,kiwi fruit,~kiwi|medium:69|61|1.14|0.52|0.029|0|3|14.66|3|8.99|34|0.31|312|92.7|4|0
avocado,avocados|whole:201;cup:150;slice:30|160|2|14.66|2.126|0|7|8.53|6.7|0.66|12|0.55|485|10|7|0
tomato,tomatoes|medium:123;large:182;small:91;slice:20;cup:180|18|0.88|0.2|0.028|0|5|3.89|1.2|2.63|10|0.27|237|13.7|42|0
carrot,carrots|medium:61;large:72;small:50;cup:128|41|0.93|0.24|0.037|0|69|9.58|2.8|4.74|33|0.3|320|5.9|835|0
broccoli|cup:91;stalk:151|34|2.82|0.37|0.039|0|33|6.64|2.6|1.7|47|0.73|316|89.2|31|0
spinach|cup:30;bunch:340|23|2.86|0.39|0.063|0|79|3.63|2.2|0.42|99|2.71|558|28.1|469|0
lettuce,iceberg lettuce|cup:72;leaf:8;head:539|14|0.9|0.14|0.018|0|10|2.97|1.2|1.97|18|0.41|141|2.8|25|0
radish,radishes|medium:4.5;cup:116|16|0.68|0.1|0.032|0|39|3.4|1.6|1.86|25|0.34|233|14.8|0|0
artichoke,artichokes,globe artichoke|medium:128;large:162|47|3.27|0.15|0.036|0|94|10.51|5.4|0.99|44|1.28|370|11.7|1|0
potato,potatoes,baked potato|medium:173;large:299;small:138|93|2.5|0.13|0.034|0|10|21.15|2.2|1.18|15|1.08|535|9.6|1|0
sweet potato,sweet potatoes|medium:130;cup:133|86|1.57|0.05|0.018|0|55|20.12|3|4.18|30|0.61|337|2.4|709|0
cucumber,cucumbers|whole:301;cup:104;slice:7|15|0.65|0.11|0.037|0|2|3.63|0.5|1.67|16|0.28|147|2.8|5|0
onion,onions|medium:110;cup:160;slice:14|40|1.1|0.1|0.042|0|4|9.34|1.7|4.24|23|0.21|146|7.4|0|0
corn,sweet corn,corn on the cob|ear:102;cup:145|86|3.27|1.35|0.325|0|15|18.7|2|6.26|2|0.52|270|6.8|9|0
peas,green peas|cup:145|81|5.42|0.4|0.071|0|5|14.45|5.7|5.67|25|1.47|244|40|38|0
celery|stalk:40;cup:101|14|0.69|0.17|0.042|0|80|2.97|1.6|1.34|40|0.2|260|3.1|22|0
mushrooms,mushroom,white mushrooms|cup:70;medium:18|22|3.09|0.34|0.05|0|5|3.26|1|1.98|3|0.5|318|2.1|0|0
cauliflower|cup:107;head:588|25|1.92|0.28|0.13|0|30|4.97|2|1.91|22|0.42|299|48.2|0|0
~garlic|clove:3;tsp:2.8|149|6.36|0.5|0.089|0|17|33.06|2.1|1|181|1.7|401|31.2|0|0
kale|cup:67|49|4.28|0.93|0.091|0|38|8.75|3.6|2.26|150|1.47|491|120|500|0
bell pepper,red pepper,red bell pepper,sweet pepper|medium:119;cup:149|31|0.99|0.3|0.027|0|4|6.03|2.1|4.2|7|0.43|211|127.7|157|0
green beans,string beans|cup:100|31|1.83|0.22|0.05|0|6|6.97|2.7|3.26|37|1.03|211|12.2|35|0
white bread,~bread|slice:25;cup:30|266|7.64|3.29|0.717|0|490|50.61|2.4|5.34|260|3.6|117|0|0|0
whole wheat bread,wheat bread,whole-wheat bread,whole grain bread|slice:32|247|12.95|3.36|0.734|0|450|41.29|6.8|5.57|161|2.47|254|0|0|0
croissant,croissants,butter croissant|medium:57;large:67;small:42|406|8.2|21|11.66|67|467|45.8|2.6|11.26|37|2.03|118|0.2|206|0
bagel,bagels,plain bagel|medium:105;large:131;small:69|257|10.13|1.61|0.3|0|430|50.5|2.2|5.05|15|4.8|90|0|0|0
~rice,white rice,cooked rice|cup:158|130|2.69|0.28|0.077|0|1|28.17|0.4|0.05|10|1.2|35|0|0|0
brown rice|cup:195|112|2.32|0.83|0.17|0|5|23.51|1.8|0.35|10|0.53|79|0|0|0
pasta,spaghetti,cooked pasta|cup:140|158|5.8|0.93|0.176|0|1|30.86|1.8|0.56|7|1.28|44|0|0|0
oatmeal,porridge,cooked oatmeal|cup:234|71|2.54|1.52|0.31|0|4|12|1.7|0.27|9|0.9|70|0|0|0
corn flakes,cornflakes|cup:28|357|7.5|0.4|0.1|0|714|84|3.3|9.5|5|28.9|168|21.4|321|0
popcorn,air-popped popcorn|cup:8|387|12.94|4.54|0.637|0|8|77.78|14.5|0.87|7|3.19|329|0|10|0
peanuts,peanut|oz:28.35;cup:146|567|25.8|49.24|6.28|0|18|16.13|8.5|4.72|92|4.58|705|0|0|0
almonds,almond|oz:28.35;cup:143;almond:1.2|579|21.15|49.93|3.8|0|1|21.55|12.5|4.35|269|3.71|733|0|0|0
walnuts,walnut|oz:28.35;cup:117|654|15.23|65.21|6.13|0|2|13.71|6.7|2.61|98|2.91|441|1.3|1|0
cashews,cashew|oz:28.35;cup:137|553|18.22|43.85|7.78|0|12|30.19|3.3|5.91|37|6.68|660|0.5|0|0
peanut butter,creamy peanut butter,smooth peanut butter|tbsp:16;cup:258|588|25.09|50.39|10.12|0|459|19.56|6|9.22|43|1.74|649|0|0|0
black beans|cup:172|132|8.86|0.54|0.139|0|1|23.71|8.7|0.32|27|2.1|355|0|0|0
lentils|cup:198|116|9.02|0.38|0.053|0|2|20.13|7.9|1.8|19|3.33|369|1.5|0|0
hummus|tbsp:15;cup:246|166|7.9|9.6|1.437|0|379|14.29|6|0.27|38|2.44|228|0|1|0
tofu,firm tofu|cup:252;slice:84;oz:28.35|144|17.27|8.72|1.261|0|14|2.78|2.3|0.6|683|2.66|237|0.2|8|0
~milk,whole milk|cup:244;fl oz:30.5|61|3.15|3.25|1.865|10|43|4.8|0|5.05|113|0.03|132|0|46|0
skim milk,nonfat milk,fat free milk,fat-free milk|cup:245;fl oz:30.6|34|3.37|0.08|0.056|2|42|4.96|0|5.09|122|0.03|156|0|61|0
lowfat milk,low fat milk,low-fat milk,1% milk|cup:244;fl oz:30.5|42|3.37|0.97|0.633|5|44|4.99|0|5.2|125|0.03|150|0|58|0
2% milk,reduced fat milk|cup:244;fl oz:30.5|50|3.3|1.98|1.257|8|47|4.8|0|5.06|120|0.02|140|0.2|55|0
cheddar,cheddar cheese,~cheese|slice:28;oz:28.35;cup:113|403|24.9|33.14|21.09|105|621|1.28|0|0.52|721|0.68|98|0|265|0
swiss cheese,~swiss|slice:28;oz:28.35;cup:132|380|26.96|27.8|17.78|93|192|5.38|0|1.32|791|0.13|77|0|220|0
provolone,provolone cheese|slice:28;oz:28.35|351|25.58|26.62|17.078|69|876|2.14|0|0.56|756|0.52|138|0|237|0
mozzarella,mozzarella cheese|oz:28.35;slice:28;cup:112|300|22.17|22.35|13.152|79|627|2.19|0|1.03|505|0.44|76|0|179|0
parmesan,parmesan cheese|tbsp:5;oz:28.35|431|38.46|28.61|17.301|88|1529|4.06|0|0.9|1184|0.82|125|0|207|0
cottage cheese,creamed cottage cheese|cup:210;tbsp:13|98|11.12|4.3|1.718|17|364|3.38|0|2.67|83|0.07|104|0|37|0
low fat cottage cheese,lowfat cottage cheese,low-fat cottage cheese,2% cottage cheese|cup:226;tbsp:14|81|10.45|2.27|1.235|12|308|4.76|0|4|91|0.16|125|0|21|0
cream cheese|tbsp:14.5;oz:28.35|342|5.93|34.24|19.292|110|321|4.07|0|3.21|98|0.38|138|0|308|0
yogurt,yoghurt,plain yogurt|cup:245;container:170|61|3.47|3.25|2.096|13|46|4.66|0|4.66|121|0.05|155|0.5|27|0
greek yogurt,nonfat greek yogurt|container:170;cup:245|59|10.19|0.39|0.117|5|36|3.6|0|3.24|110|0.07|141|0|1|0
butter|tbsp:14.2;pat:5;stick:113;tsp:4.7|717|0.85|81.11|51.368|215|643|0.06|0|0.06|24|0.02|24|0|684|0
olive oil|tbsp:13.5;tsp:4.5;cup:216|884|0|100|13.808|0|2|0|0|0|1|0.56|1|0|0|0
eggs,egg,large egg|large:50;medium:44;small:38;extra large:56;jumbo:63|143|12.56|9.51|3.126|372|142|0.72|0|0.37|56|1.75|138|0|160|0
ice cream,vanilla ice cream|half cup:66;cup:132;scoop:66|207|3.5|11|6.79|44|80|23.6|0.7|21.22|128|0.09|199|0.6|118|0
hamburger,hamburgers,burger|sandwich:105|254|12.6|9.4|3.5|30|495|30.6|1.2|6.2|126|2.6|199|0|6|0
ground beef,hamburger patty|patty:85;oz:28.35|254|25.75|16.29|6.21|89|75|0|0|0|24|2.6|364|0|0|0
~chicken,chicken breast|breast:172;oz:28.35;cup:140|165|31.02|3.57|1.01|85|74|0|0|0|15|1.04|256|0|6|0
~turkey,turkey breast,roast turkey|slice:28;oz:28.35;cup:140|135|30.13|0.74|0.24|69|99|0|0|0|10|0.71|293|0|0|0
~salmon|fillet:178;oz:28.35|206|22.1|12.35|2.5|63|61|0|0|0|15|0.34|384|3.7|69|0
~tuna,canned tuna,tuna fish|can:165;oz:28.35|116|25.51|0.82|0.234|30|338|0|0|0|11|1.53|237|0|17|0
~shrimp|oz:28.35;large:5.5|99|23.98|0.28|0.079|189|111|0.2|0|0|70|0.51|259|0|0|0
bacon|slice:8|541|37.04|41.78|13.747|110|1717|1.43|0|0|11|1.44|565|0|11|0
pizza,cheese pizza|slice:107|266|11.39|9.69|4.478|17|598|33.33|2.3|3.58|201|2.45|172|0|68|0
french fries,fries|medium:117;small:71;large:154|312|3.43|14.73|2.27|0|210|41.44|3.8|0.3|18|0.81|579|4.7|0|0
potato chips,crisps|oz:28.35|536|7|34.6|3.4|0|525|53|4.4|0.3|24|1.6|1275|19.7|0|0
~sugar,white sugar,table sugar|tsp:4.2;tbsp:12.5;cup:200|387|0|0|0|0|1|99.98|0|99.8|1|0.05|2|0|0|0
~honey|tbsp:21;tsp:7|304|0.3|0|0|0|4|82.4|0.2|82.12|6|0.42|52|0.5|0|0
~chocolate,milk chocolate,chocolate bar|bar:44;oz:28.35|535|7.65|29.66|18.509|23|79|59.4|3.4|51.5|189|2.35|372|0|50|20
dark chocolate|oz:28.35;bar:101|598|7.79|42.63|24.489|3|20|45.9|10.9|23.99|73|11.9|715|0|2|80
ketchup|tbsp:17;packet:10|101|1.04|0.1|0.028|0|907|27.4|0.3|21.27|15|0.35|281|4.1|26|0
mayonnaise,~mayo|tbsp:13.8|680|0.96|74.85|11.742|42|635|0.57|0|0.57|8|0.21|20|0|54|0
orange juice,oj,fresh orange juice|cup:248;glass:248;fl oz:31|45|0.7|0.2|0.024|0|1|10.4|0.2|8.4|11|0.2|200|50|10|0
apple juice|cup:248;glass:248;fl oz:31|46|0.1|0.13|0.022|0|4|11.3|0.2|9.62|8|0.12|101|0.9|1|0
~coffee,brewed coffee,black coffee|cup:237;fl oz:29.6|1|0.12|0.02|0.002|0|2|0|0|0|2|0.01|49|0|0|40
espresso|shot:30;fl oz:29.6|9|0.12|0.18|0.093|0|14|1.67|0|0|2|0.13|115|0.2|0|212
~tea,black tea,brewed tea|cup:237;fl oz:29.6|1|0|0|0.002|0|3|0.3|0|0|0|0.02|37|0|0|20
~cola,~soda,~coke,soft drink|can:368;cup:246;fl oz:30.7|38|0|0.02|0|0|4|9.56|0|8.97|2|0.11|2|0|0|9.6
energy drink,~red bull|can:250;fl oz:30|45|0.25|0.08|0|0|84|11|0|10|13|0.04|8|0|0|32
~beer|can:356;bottle:356;pint:473;fl oz:29.7|43|0.46|0|0|0|4|3.55|0|0|4|0.02|27|0|0|0
~wine,red wine|glass:147;fl oz:29.4|85|0.07|0|0|0|4|2.61|0|0.62|8|0.46|127|0|0|0
~vodka|shot:42;cup:222;fl oz:27.8|231|0|0|0|0|1|0|0|0|0|0.04|1|0|0|0
";

const N: usize = 15;

/// aliases | label | unit | FDA daily value (0 = none)
const NUTRIENTS: &[(&[&str], &str, &str, f64)] = &[
    (&["calories", "calorie", "kcal", "energy", "kilocalories", "food energy"], "calories", "kcal", 2000.0),
    (&["protein", "proteins"], "protein", "g", 50.0),
    (&["fat", "total fat", "fats"], "total fat", "g", 78.0),
    (&["saturated fat", "sat fat", "saturated fats"], "saturated fat", "g", 20.0),
    (&["cholesterol"], "cholesterol", "mg", 300.0),
    (&["sodium"], "sodium", "mg", 2300.0),
    (&["carbohydrate", "carbohydrates", "carbs", "carb", "total carbohydrate", "total carbohydrates"], "carbohydrate", "g", 275.0),
    (&["fiber", "fibre", "dietary fiber", "dietary fibre"], "dietary fiber", "g", 28.0),
    (&["sugar", "sugars", "total sugars"], "sugars", "g", 0.0),
    (&["calcium"], "calcium", "mg", 1300.0),
    (&["iron"], "iron", "mg", 18.0),
    (&["potassium"], "potassium", "mg", 4700.0),
    (&["vitamin c", "vit c", "ascorbic acid"], "vitamin C", "mg", 90.0),
    (&["vitamin a", "vit a"], "vitamin A", "µg", 900.0),
    (&["caffeine"], "caffeine", "mg", 0.0),
];

struct Food {
    names: Vec<(&'static str, bool)>,
    units: Vec<(&'static str, f64)>,
    v: [f64; N],
}

fn foods() -> &'static Vec<Food> {
    static F: OnceLock<Vec<Food>> = OnceLock::new();
    F.get_or_init(|| {
        FOODS
            .lines()
            .filter(|r| !r.trim().is_empty())
            .map(|r| {
                let f: Vec<&str> = r.split('|').collect();
                let names = f[0].split(',').map(|n| (n.trim_start_matches('~'), n.starts_with('~'))).collect();
                let units = f[1]
                    .split(';')
                    .filter_map(|u| {
                        let (a, b) = u.split_once(':')?;
                        Some((a, b.parse().ok()?))
                    })
                    .collect();
                let mut v = [0.0; N];
                for (k, x) in f[2..].iter().enumerate().take(N) {
                    v[k] = x.trim().parse().unwrap_or(0.0);
                }
                Food { names, units, v }
            })
            .collect()
    })
}

/// Find a food by name; the flag says the matched name is ambiguous bare.
fn find_food(s: &str) -> Option<(&'static Food, bool)> {
    let s = s.trim().trim_start_matches("the ").trim();
    let lookup = |n: &str| {
        foods().iter().find_map(|f| f.names.iter().find(|x| x.0 == n).map(|x| (f, x.1)))
    };
    let mut cands = vec![s.to_string()];
    for adj in ["fresh ", "raw ", "plain ", "whole ", "medium ", "ripe ", "regular "] {
        if let Some(r) = s.strip_prefix(adj) {
            cands.push(r.to_string());
        }
    }
    for c in cands {
        let c = c.as_str();
        if let Some(r) = lookup(c) {
            return Some(r);
        }
        for (suf, rep) in [("ies", "y"), ("es", ""), ("s", "")] {
            if let Some(stem) = c.strip_suffix(suf) {
                if let Some(r) = lookup(&format!("{stem}{rep}")) {
                    return Some(r);
                }
            }
        }
    }
    None
}

fn nutrient(s: &str) -> Option<usize> {
    let s = s.trim().trim_end_matches(" content").trim_start_matches("total ").trim();
    NUTRIENTS
        .iter()
        .position(|n| n.0.contains(&s))
        .or_else(|| NUTRIENTS.iter().position(|n| n.0.contains(&format!("total {s}").as_str())))
}

/// Round to three significant figures (whole numbers from 100 up).
fn sig(v: f64) -> String {
    if v == 0.0 || !v.is_finite() {
        return "0".into();
    }
    if v.abs() >= 100.0 {
        return format!("{}", v.round() as i64);
    }
    let d = (2 - v.abs().log10().floor() as i32).max(0) as usize;
    let s = format!("{:.*}", d, v);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

fn amount_text(i: usize, v: f64) -> String {
    let (_, _, unit, _) = NUTRIENTS[i];
    if i == 0 {
        format!("{} kcal", v.round())
    } else {
        format!("{} {unit}", sig(v))
    }
}

struct Item {
    food: &'static Food,
    grams: f64,
    desc: String,
    ambiguous: bool,
    explicit: bool,
}

fn number(s: &str) -> Option<f64> {
    let words = [("a", 1.0), ("an", 1.0), ("one", 1.0), ("two", 2.0), ("three", 3.0), ("four", 4.0), ("five", 5.0), ("six", 6.0), ("seven", 7.0), ("eight", 8.0), ("nine", 9.0), ("ten", 10.0), ("half", 0.5), ("half a", 0.5), ("half an", 0.5), ("a half", 0.5), ("a quarter", 0.25), ("quarter", 0.25), ("a dozen", 12.0), ("dozen", 12.0)];
    if let Some(w) = words.iter().find(|w| w.0 == s) {
        return Some(w.1);
    }
    if let Some(c) = re!(r"^(\d+) (\d+)/(\d+)$").captures(s) {
        return Some(c[1].parse::<f64>().ok()? + c[2].parse::<f64>().ok()? / c[3].parse::<f64>().ok()?);
    }
    if let Some((a, b)) = s.split_once('/') {
        return Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?);
    }
    s.parse().ok()
}

fn food_unit(f: &Food, u: &str) -> Option<f64> {
    f.units.iter().find(|x| x.0 == u).map(|x| x.1)
}

/// Grams in one `u` of food `f`, with the label to show.
fn resolve_unit(f: &'static Food, u: &str) -> Option<(String, f64)> {
    let u = u.trim().trim_end_matches('.').trim();
    let fl = food_unit(f, "fl oz");
    let tbsp = food_unit(f, "tbsp").or_else(|| food_unit(f, "cup").map(|c| c / 16.0));
    let r = match u {
        "g" | "gram" | "grams" | "gm" => ("g", 1.0),
        "kg" | "kilogram" | "kilograms" => ("kg", 1000.0),
        "lb" | "lbs" | "pound" | "pounds" => ("lb", 453.592),
        "oz" | "ounce" | "ounces" => match fl {
            Some(g) => ("fl oz", g),
            None => ("oz", food_unit(f, "oz").unwrap_or(28.3495)),
        },
        "fl oz" | "fluid ounce" | "fluid ounces" | "fl. oz" => ("fl oz", fl?),
        "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" => ("mL", fl? / 29.5735),
        "l" | "liter" | "liters" | "litre" | "litres" => ("L", fl? / 0.0295735),
        "tbsp" | "tablespoon" | "tablespoons" | "tbs" | "tbsps" => ("tbsp", tbsp?),
        "tsp" | "teaspoon" | "teaspoons" | "tsps" => ("tsp", food_unit(f, "tsp").or(tbsp.map(|t| t / 3.0))?),
        "glass" | "glasses" => ("glass", food_unit(f, "glass").or(food_unit(f, "cup"))?),
        "serving" | "servings" | "piece" | "pieces" | "portion" | "portions" | "bowl" | "bowls" | "item" | "each" | "helping" => {
            let (n, g) = f.units.first()?;
            return Some((n.to_string(), *g));
        }
        "whole" => {
            let (n, g) = f.units.iter().find(|x| x.0 == "whole").or(f.units.first())?;
            return Some((n.to_string(), *g));
        }
        _ => {
            let mut cands = vec![u.to_string()];
            if u == "leaves" {
                cands.push("leaf".into());
            }
            for (suf, rep) in [("ies", "y"), ("es", ""), ("s", "")] {
                if let Some(stem) = u.strip_suffix(suf) {
                    cands.push(format!("{stem}{rep}"));
                }
            }
            if u == "half cup" || u == "half a cup" {
                cands.push("half cup".into());
            }
            for c in cands {
                if let Some(g) = food_unit(f, &c) {
                    return Some((c, g));
                }
            }
            return None;
        }
    };
    Some((r.0.to_string(), r.1))
}

fn plural_unit(u: &str, amt: f64) -> String {
    if (amt - 1.0).abs() < 1e-9 || amt < 1.0 || ["g", "kg", "lb", "oz", "fl oz", "mL", "L", "tbsp", "tsp", "small", "medium", "large", "extra large", "jumbo", "whole", "half cup"].contains(&u) {
        return u.to_string();
    }
    if let Some(s) = u.strip_suffix('y') {
        return format!("{s}ies");
    }
    if u == "leaf" {
        return "leaves".into();
    }
    if u.ends_with('h') || u.ends_with('s') {
        return format!("{u}es");
    }
    format!("{u}s")
}

fn parse_item(s: &str) -> Option<Item> {
    let s = s.trim().trim_end_matches(['.', '?']).trim();
    let s = re!(r"\b(\d+(?:\.\d+)?)(g|kg|oz|lb|lbs|ml|l)\b").replace_all(s, "$1 $2").to_string();
    let s = s.replace("fl. oz", "fl oz").replace("oz.", "oz");
    let s = s.trim().trim_start_matches("of ").trim();
    let am = re!(r"^(\d+ \d+/\d+|\d+/\d+|\d+(?:\.\d+)?|\.\d+|half an?|a half|a quarter|a dozen|half|quarter|dozen|an?|one|two|three|four|five|six|seven|eight|nine|ten) (.+)$").captures(s);
    let (amt, rest, has_amt) = match &am {
        Some(c) => (number(&c[1])?, c[2].to_string(), true),
        None => (1.0, s.to_string(), false),
    };
    let words: Vec<&str> = rest.split_whitespace().collect();
    for k in 0..=words.len().saturating_sub(1).min(3) {
        let unit = words[..k].join(" ");
        let food_str = words[k..].join(" ");
        let food_str = food_str.trim_start_matches("of ").trim();
        if food_str.is_empty() {
            continue;
        }
        let Some((food, amb)) = find_food(food_str) else { continue };
        let (ulabel, g) = if k == 0 { let (n, g) = food.units.first()?; (n.to_string(), *g) } else {
            match resolve_unit(food, &unit) { Some(x) => x, None => continue }
        };
        let name = food.names[0].0;
        let amt_s = if (amt - 0.5).abs() < 1e-9 { "1/2".to_string() } else if (amt - 0.25).abs() < 1e-9 { "1/4".to_string() } else { sig(amt) };
        let desc = format!("{amt_s} {} {name}", plural_unit(&ulabel, amt));
        return Some(Item { food, grams: amt * g, desc, ambiguous: amb, explicit: has_amt || k > 0 });
    }
    None
}

fn totals(items: &[Item]) -> [f64; N] {
    let mut t = [0.0; N];
    for it in items {
        for (k, x) in t.iter_mut().enumerate() {
            *x += it.food.v[k] * it.grams / 100.0;
        }
    }
    t
}

fn grams(items: &[Item]) -> f64 {
    items.iter().map(|i| i.grams).sum()
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn note() -> Pod {
    pod("Note", "Note", text("Approximate values after USDA FoodData Central; % daily values use the FDA reference amounts for a 2,000-calorie diet. Actual foods vary with variety, size and preparation."))
}

fn summary(t: &[f64; N]) -> String {
    format!("{} kcal, {} g protein, {} g carbohydrate, {} g fat", t[0].round(), sig(t[1]), sig(t[6]), sig(t[2]))
}

fn label_table(t: &[f64; N]) -> Content {
    let rows = (0..N)
        .filter(|&k| k != 14 || t[14] > 0.0)
        .map(|k| {
            let (_, lab, _, dv) = NUTRIENTS[k];
            let pct = if dv > 0.0 { format!("{}%", (t[k] / dv * 100.0).round()) } else { String::new() };
            let lab = if k == 3 || k == 7 || k == 8 { format!("  {lab}") } else { lab.to_string() };
            vec![text(lab), text(amount_text(k, t[k])), text(pct)]
        })
        .collect();
    table(Some(vec!["nutrient", "amount", "% daily value"]), rows)
}

/// Split "a + b + c" (a meal) into items.
fn parse_group(s: &str) -> Option<Vec<Item>> {
    let parts: Vec<&str> = re!(r"\s*\+\s*|\s+plus\s+|\s+with\s+").split(s).collect();
    parts.iter().map(|p| parse_item(p)).collect()
}

fn group_desc(g: &[Item]) -> String {
    g.iter().map(|i| i.desc.clone()).collect::<Vec<_>>().join(" + ")
}

/// A nutrient amount ("vitamin c in an apple") or a daily value.
fn quantity(s: &str) -> Option<(f64, &'static str, String)> {
    if let Some((i, v, desc)) = reference(s) {
        return Some((v, NUTRIENTS[i].2, desc));
    }
    let m = re!(r"^(.+?) (?:in|of) (.+)$").captures(s)?;
    let i = nutrient(&m[1])?;
    let g = parse_group(&m[2])?;
    Some((totals(&g)[i], NUTRIENTS[i].2, format!("{} in {}", NUTRIENTS[i].1, group_desc(&g))))
}

// ---------------------------------------------------------------------------
// daily values and dietary reference intakes
// ---------------------------------------------------------------------------

/// nutrient label | kind | unit | rows "sex age-from age-to value" (sex B/M/F; ages in years, 999 = no upper bound)
const DRI: &[(&str, &str, &str, &str)] = &[
    ("protein", "RDA", "g", "B 1 3 13;B 4 8 19;B 9 13 34;M 14 18 52;F 14 18 46;M 19 999 56;F 19 999 46"),
    ("carbohydrate", "RDA", "g", "B 1 999 130"),
    ("dietary fiber", "AI", "g", "B 1 3 19;B 4 8 25;M 9 13 31;F 9 13 26;M 14 50 38;F 14 18 26;F 19 50 25;M 51 999 30;F 51 999 21"),
    ("calcium", "RDA", "mg", "B 1 3 700;B 4 8 1000;B 9 18 1300;B 19 50 1000;M 51 70 1000;F 51 70 1200;B 71 999 1200"),
    ("iron", "RDA", "mg", "B 1 3 7;B 4 8 10;B 9 13 8;M 14 18 11;F 14 18 15;M 19 999 8;F 19 50 18;F 51 999 8"),
    ("vitamin C", "RDA", "mg", "B 1 3 15;B 4 8 25;B 9 13 45;M 14 18 75;F 14 18 65;M 19 999 90;F 19 999 75"),
    ("vitamin A", "RDA", "µg", "B 1 3 300;B 4 8 400;B 9 13 600;M 14 999 900;F 14 999 700"),
    ("sodium", "AI", "mg", "B 1 3 800;B 4 8 1000;B 9 13 1200;B 14 999 1500"),
    ("potassium", "AI", "mg", "B 1 3 2000;B 4 8 2300;M 9 13 2500;F 9 13 2300;M 14 18 3000;F 14 18 2300;M 19 999 3400;F 19 999 2600"),
    ("vitamin D", "RDA", "µg", "B 1 70 15;B 71 999 20"),
    ("magnesium", "RDA", "mg", "B 1 3 80;B 4 8 130;B 9 13 240;M 14 18 410;F 14 18 360;M 19 30 400;F 19 30 310;M 31 999 420;F 31 999 320"),
    ("zinc", "RDA", "mg", "B 1 3 3;B 4 8 5;B 9 13 8;M 14 999 11;F 14 18 9;F 19 999 8"),
    ("vitamin B12", "RDA", "µg", "B 1 3 0.9;B 4 8 1.2;B 9 13 1.8;B 14 999 2.4"),
    ("folate", "RDA", "µg DFE", "B 1 3 150;B 4 8 200;B 9 13 300;B 14 999 400"),
];

const EXTRA_NUTRIENTS: &[(&[&str], &str)] = &[
    (&["vitamin d", "vit d"], "vitamin D"),
    (&["magnesium"], "magnesium"),
    (&["zinc"], "zinc"),
    (&["vitamin b12", "b12", "vit b12", "cobalamin"], "vitamin B12"),
    (&["folate", "folic acid"], "folate"),
];

fn dri_rows(label: &str) -> Option<(&'static str, &'static str, Vec<(char, f64, f64, f64)>)> {
    let d = DRI.iter().find(|d| d.0 == label)?;
    let rows = d
        .3
        .split(';')
        .filter_map(|r| {
            let p: Vec<&str> = r.split_whitespace().collect();
            Some((p[0].chars().next()?, p[1].parse().ok()?, p[2].parse().ok()?, p[3].parse().ok()?))
        })
        .collect();
    Some((d.1, d.2, rows))
}

fn dri_value(rows: &[(char, f64, f64, f64)], age: f64, male: bool) -> Option<f64> {
    rows.iter()
        .find(|r| (r.0 == 'B' || (r.0 == 'M') == male) && age >= r.1 && age < r.2 + 1.0)
        .map(|r| r.3)
}

/// "iron DRI for 65 year old male", "sodium RDA", "protein daily value",
/// "daily value for vitamin C", "fat RDA * 1 week".
/// Returns (nutrient index or usize::MAX for extra nutrients, value, description).
fn reference(s: &str) -> Option<(usize, f64, String)> {
    let kinds = r"(rda|rdas|dri|dris|ai|daily value|daily values|dv|%dv|recommended daily (?:allowance|intake|amount|value)|recommended dietary allowance|recommended (?:intake|amount)|daily (?:allowance|intake|requirement|amount|recommended intake)|daily recommended (?:intake|amount|allowance)|requirement|daily requirement)";
    let who_re = r"(?: (?:for|of) (?:an? |the )?(.+?))?(?: (?:\*|times|x) (\d+(?:\.\d+)?) (days?|weeks?|months?|years?))?";
    let pat1 = format!(r"^(?:the )?(.+?) {kinds}{who_re}$");
    let pat2 = format!(r"^(?:the )?{kinds} (?:of|for) (.+?)(?: for (?:an? |the )?(.+?))?(?: (?:\*|times|x) (\d+(?:\.\d+)?) (days?|weeks?|months?|years?))?$");
    static R1: OnceLock<Regex> = OnceLock::new();
    static R2: OnceLock<Regex> = OnceLock::new();
    let r1 = R1.get_or_init(|| Regex::new(&pat1).unwrap());
    let r2 = R2.get_or_init(|| Regex::new(&pat2).unwrap());
    let (nut, kind, who, k, per) = if let Some(m) = r1.captures(s) {
        (m[1].to_string(), m[2].to_string(), m.get(3).map(|x| x.as_str().to_string()), m.get(4).map(|x| x.as_str().to_string()), m.get(5).map(|x| x.as_str().to_string()))
    } else {
        let m = r2.captures(s)?;
        (m[2].to_string(), m[1].to_string(), m.get(3).map(|x| x.as_str().to_string()), m.get(4).map(|x| x.as_str().to_string()), m.get(5).map(|x| x.as_str().to_string()))
    };
    let idx = nutrient(&nut);
    let label: &str = match idx {
        Some(i) => NUTRIENTS[i].1,
        None => EXTRA_NUTRIENTS.iter().find(|e| e.0.contains(&nut.as_str()))?.1,
    };
    let days = match (k, per) {
        (Some(k), Some(p)) => k.parse::<f64>().ok()? * if p.starts_with("week") { 7.0 } else if p.starts_with("month") { 30.4375 } else if p.starts_with("year") { 365.25 } else { 1.0 },
        _ => 1.0,
    };
    // who: "65 year old male", "30 year old female", "women", "adult men"
    let (age, sex) = match &who {
        Some(w) => {
            let a = re!(r"(\d+(?:\.\d+)?)[- ]?(?:years?|yrs?|yo|y)?[- ]?(?:old)?").captures(w).and_then(|c| c[1].parse::<f64>().ok());
            let male = if re!(r"\b(male|man|men|boy|boys)\b").is_match(w) { Some(true) } else if re!(r"\b(female|woman|women|girl|girls)\b").is_match(w) { Some(false) } else { None };
            if a.is_none() && male.is_none() {
                return None;
            }
            (a, male)
        }
        None => (None, None),
    };
    let dv_kind = kind.contains("daily value") || kind == "dv" || kind == "%dv";
    let dri = if dv_kind { None } else { dri_rows(label) };
    let fda = idx.map(|i| NUTRIENTS[i].3).filter(|&d| d > 0.0);
    let per_text = if days == 1.0 { "per day".to_string() } else { format!("for {} days", sig(days)) };
    if let Some((dkind, unit, rows)) = &dri {
        let (value, who_text) = match (age, sex) {
            (Some(a), Some(m)) => (dri_value(rows, a, m)?, format!("{} year old {}", sig(a), if m { "male" } else { "female" })),
            (Some(a), None) => {
                let (x, y) = (dri_value(rows, a, true)?, dri_value(rows, a, false)?);
                if x != y {
                    return Some((idx.unwrap_or(usize::MAX), x * days, format!("{label} {dkind} | age {}: males {} {unit}, females {} {unit} {per_text}", sig(a), sig(x * days), sig(y * days))));
                }
                (x, format!("age {}", sig(a)))
            }
            (None, Some(m)) => (dri_value(rows, 30.0, m)?, format!("adult {}", if m { "men" } else { "women" })),
            (None, None) => {
                let (x, y) = (dri_value(rows, 30.0, true)?, dri_value(rows, 30.0, false)?);
                if x != y {
                    return Some((idx.unwrap_or(usize::MAX), x * days, format!("{label} {dkind} | adults 19–50: men {} {unit}, women {} {unit} {per_text}", sig(x * days), sig(y * days))));
                }
                (x, "adults".to_string())
            }
        };
        return Some((idx.unwrap_or(usize::MAX), value * days, format!("{label} {dkind} | {who_text}: {} {unit} {per_text}", sig(value * days))));
    }
    let dv = fda?;
    let i = idx?;
    Some((i, dv * days, format!("{label} daily value | {} {per_text}", amount_text(i, dv * days))))
}

fn reference_answer(s: &str) -> Option<Answer> {
    let (i, v, desc) = reference(s)?;
    let (head, body) = desc.split_once(" | ")?;
    let short = body.rsplit_once(": ").map(|x| x.1).unwrap_or(body).to_string();
    let short = if body.contains(", ") && body.contains("men") { body.split_once(": ").map(|x| x.1.to_string()).unwrap_or(short) } else { short };
    let mut pods = vec![input(head.to_string()), primary(pod("Result", "Recommended amount", text(short)))];
    let label = if i < N { NUTRIENTS[i].1.to_string() } else { head.split(' ').take_while(|w| !["RDA", "AI", "daily"].contains(w)).collect::<Vec<_>>().join(" ") };
    if let Some((kind, unit, rows)) = dri_rows(label.as_str()) {
        let groups = [(1.0, "children 1–3"), (4.0, "children 4–8"), (9.0, "9–13"), (14.0, "14–18"), (19.0, "19–30"), (31.0, "31–50"), (51.0, "51–70"), (71.0, "71+")];
        let t = groups
            .iter()
            .filter_map(|(a, g)| Some(vec![text(*g), text(format!("{} {unit}", sig(dri_value(&rows, *a, true)?))), text(format!("{} {unit}", sig(dri_value(&rows, *a, false)?)))]))
            .collect();
        pods.push(pod("Table", &format!("{kind} by life stage (per day)"), table(Some(vec!["age", "males", "females"]), t)));
        if label == "sodium" {
            pods.push(pod("Limit", "Chronic disease risk reduction intake", text("adults should reduce intake if above 2,300 mg/day (ages 14+); 1,200, 1,500 and 1,800 mg/day for ages 1–3, 4–8 and 9–13")));
        }
    }
    if i < N && NUTRIENTS[i].3 > 0.0 {
        pods.push(pod("DV", "FDA daily value (food labels)", text(amount_text(i, NUTRIENTS[i].3))));
    }
    if label == "total fat" {
        pods.push(pod("AMDR", "Acceptable macronutrient distribution range", text("20–35% of energy for adults (25–35% ages 4–18, 30–40% ages 1–3); no RDA is set for fat")));
    }
    let _ = v;
    pods.push(pod("Source", "Source", text("Dietary Reference Intakes, US National Academies; FDA daily values (2016 label rule)")));
    Some(Answer::new("dietary reference", pods))
}

// ---------------------------------------------------------------------------
// answers
// ---------------------------------------------------------------------------

fn meal_answer(nut: Option<usize>, g: Vec<Item>) -> Answer {
    let t = totals(&g);
    let desc = group_desc(&g);
    let gr = grams(&g);
    let mut pods = vec![input(match nut {
        Some(i) => format!("{} in {desc}", NUTRIENTS[i].1),
        None => desc.clone(),
    })];
    match nut {
        Some(i) => {
            pods.push(primary(pod("Result", &format!("Amount of {}", NUTRIENTS[i].1), text(amount_text(i, t[i])))));
            if NUTRIENTS[i].3 > 0.0 {
                pods.push(pod("DV", "Percent daily value", text(format!("{}%", (t[i] / NUTRIENTS[i].3 * 100.0).round()))));
            }
        }
        None => pods.push(primary(pod("Result", "Nutrition facts", text(summary(&t))))),
    }
    if g.len() > 1 {
        let cols = nut.map_or(vec![0, 1, 6, 2], |i| vec![i]);
        let mut header = vec!["food", "mass"];
        header.extend(cols.iter().map(|&k| NUTRIENTS[k].1));
        let mut rows: Vec<Vec<Content>> = g
            .iter()
            .map(|it| {
                let mut r = vec![text(it.desc.clone()), text(format!("{} g", sig(it.grams)))];
                r.extend(cols.iter().map(|&k| text(amount_text(k, it.food.v[k] * it.grams / 100.0))));
                r
            })
            .collect();
        let mut tot = vec![text("total"), text(format!("{} g", sig(gr)))];
        tot.extend(cols.iter().map(|&k| text(amount_text(k, t[k]))));
        rows.push(tot);
        pods.push(pod("Items", "Breakdown", table(Some(header), rows)));
    }
    pods.push(pod("Label", &format!("Nutrition facts ({} g)", sig(gr)), label_table(&t)));
    pods.push(note());
    Answer::new("nutrition", pods)
}

fn compare_answer(nut: Option<usize>, groups: Vec<Vec<Item>>) -> Answer {
    let ts: Vec<[f64; N]> = groups.iter().map(|g| totals(g)).collect();
    let names: Vec<String> = groups.iter().map(|g| group_desc(g)).collect();
    let key = nut.unwrap_or(0);
    let main = names.iter().zip(&ts).map(|(n, t)| format!("{n}: {}", amount_text(key, t[key]))).collect::<Vec<_>>().join("; ");
    let mut pods = vec![
        input(match nut {
            Some(i) => format!("{} | {}", NUTRIENTS[i].1, names.join(" vs ")),
            None => names.join(" vs "),
        }),
        primary(pod("Result", &format!("Amount of {}", NUTRIENTS[key].1), text(main))),
    ];
    if ts.len() == 2 && ts[1][key] > 0.0 && ts[0][key] > 0.0 {
        pods.push(pod("Ratio", "Ratio", text(format!("{} : 1", sig(ts[0][key] / ts[1][key])))));
    }
    let mut header = vec!["nutrient".to_string()];
    header.extend(names.iter().cloned());
    let rows = (0..N)
        .filter(|&k| k != 14 || ts.iter().any(|t| t[14] > 0.0))
        .map(|k| {
            let mut r = vec![text(NUTRIENTS[k].1)];
            r.extend(ts.iter().map(|t| text(amount_text(k, t[k]))));
            r
        })
        .collect();
    let mut mass = vec![text("mass")];
    mass.extend(groups.iter().map(|g| text(format!("{} g", sig(grams(g))))));
    let mut rows: Vec<Vec<Content>> = rows;
    rows.insert(0, mass);
    pods.push(pod("Table", "Nutrition comparison", table(Some(header.iter().map(|s| s.as_str()).collect()), rows)));
    pods.push(note());
    Answer::new("nutrition comparison", pods)
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().to_string();
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let mut l = l.as_str();
    for p in ["how much ", "how many ", "what is the ", "what's the ", "what are the ", "what is ", "amount of ", "the amount of ", "total ", "the "] {
        if let Some(r) = l.strip_prefix(p) {
            l = r;
        }
    }
    let l = l.trim_end_matches(" are there").trim_end_matches(" is there");

    if let Some(a) = reference_answer(l) {
        return Some(a);
    }

    // "daily value for vitamin C / vitamin c in an apple"
    if let Some((a, b)) = l.split_once(" / ") {
        let (x, ux, dx) = quantity(a.trim())?;
        let (y, uy, dy) = quantity(b.trim())?;
        if ux != uy || y == 0.0 {
            return None;
        }
        return Some(Answer::new("nutrition ratio", vec![
            input(format!("({}) / ({})", dx.replace(" | ", ": "), dy.replace(" | ", ": "))),
            primary(pod("Result", "Result", text(sig(x / y)))),
            pod("Values", "Values", text(format!("{} {ux} / {} {uy}", sig(x), sig(y)))),
        ]));
    }

    // what is being asked: a nutrient, the full label, or a comparison
    let mut nut = None;
    let mut label = false;
    let mut compare = false;
    let mut rest = l.to_string();
    if let Some(m) = re!(r"^(.+?) (?:are |is )?(?:there )?(?:in|of|for|contained in) (.+)$").captures(l) {
        if let Some(i) = nutrient(&m[1]) {
            nut = Some(i);
            rest = m[2].to_string();
        }
    }
    if nut.is_none() {
        if let Some(m) = re!(r"^(?:nutrition|nutritional|nutrient|nutrients|nutrition facts|nutrition label|nutritional (?:value|values|information|info|facts|content)|nutrition (?:value|values|information|info|content)|calorie count)(?: (?:of|for|in))? (.+)$").captures(l) {
            label = true;
            rest = m[1].to_string();
        } else if let Some(m) = re!(r"^(.+?) (?:nutrition|nutritional|nutrient|nutrients)(?: facts| label| value| values| information| info| content| data)?$").captures(l) {
            label = true;
            rest = m[1].to_string();
        } else if let Some(m) = re!(r"^(.+?) (calories|kcal|protein|fat|carbs|sodium|fiber|fibre|sugar|cholesterol|caffeine)(?: content)?$").captures(l) {
            nut = nutrient(&m[2]);
            rest = m[1].to_string();
        }
    }
    if let Some(r) = rest.strip_prefix("compare ") {
        compare = true;
        rest = r.to_string();
    }
    let rest = rest.trim();

    // comparisons: "apple vs orange", "compare A, B", "N in A vs B"
    let vs: Vec<&str> = re!(r"\s+(?:vs\.?|versus|compared to|compared with)\s+").split(rest).collect();
    if vs.len() > 1 || (compare && (rest.contains(", ") || rest.contains(" and "))) {
        let parts: Vec<&str> = if vs.len() > 1 { vs } else { re!(r",\s*(?:and\s+)?|\s+and\s+").split(rest).collect() };
        let groups: Vec<Vec<Item>> = parts.iter().map(|p| {
            // allow "sodium in white bread vs sodium in wheat bread"
            let p = match (nut, re!(r"^(.+?) (?:in|of) (.+)$").captures(p)) {
                (Some(i), Some(m)) if nutrient(&m[1]) == Some(i) => m.get(2).map_or(*p, |x| x.as_str()),
                _ => p,
            };
            parse_group(p)
        }).collect::<Option<_>>()?;
        return Some(compare_answer(nut, groups));
    }

    // a meal: "a + b + c"; a list "a, b, c"
    if re!(r"\s\+\s|\s+plus\s+").is_match(rest) {
        let g = parse_group(rest)?;
        return Some(meal_answer(nut, g));
    }
    if rest.contains(", ") {
        let parts: Vec<&str> = re!(r",\s*(?:and\s+)?").split(rest).collect();
        let items: Vec<Item> = parts.iter().map(|p| parse_item(p)).collect::<Option<_>>()?;
        if nut.is_some() {
            return Some(compare_answer(nut, items.into_iter().map(|i| vec![i]).collect()));
        }
        return Some(meal_answer(None, items));
    }

    // one food
    let it = parse_item(rest)?;
    if nut.is_none() && !label && !it.explicit && it.ambiguous {
        return None;
    }
    Some(meal_answer(nut, vec![it]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_parse() {
        for r in FOODS.lines().filter(|r| !r.trim().is_empty()) {
            assert_eq!(r.split('|').count(), 2 + N, "{r}");
        }
        for d in DRI {
            assert!(dri_rows(d.0).is_some());
        }
    }
}
