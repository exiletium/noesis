//! Sports: regulation equipment and playing areas, professional and college
//! teams, stadiums and arenas, and Olympic medal counts.

use crate::engine::*;
use crate::print::fmt_float;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | sport | mass kg (min-max) | diameter m | length m | width m | height m | rules
const EQUIPMENT: &str = "\
bowling ball,ten-pin bowling ball,ten pin bowling ball|ten-pin bowling|4.5-7.26|0.2183|||| at most 16 lb (7.26 kg); diameter 8.5–8.595 in (21.6–21.83 cm)
cricket ball|cricket|0.1559-0.163|0.0718|||| men's ball 155.9–163 g (5.5–5.75 oz); circumference 22.4–22.9 cm
baseball,baseballs|baseball|0.142-0.149|0.0737|||| 5–5.25 oz; circumference 9–9.25 in; 108 double stitches
softball,softballs|softball|0.178-0.198|0.097|||| 12-inch fastpitch ball: 6.25–7 oz
soccer ball,football (soccer),association football ball,size 5 soccer ball|association football|0.41-0.45|0.22|||| size 5: circumference 68–70 cm, 410–450 g, pressure 0.6–1.1 atm
american football,nfl football,gridiron football|American football|0.397-0.425||0.28||| 14–15 oz; 11–11.25 in long; long circumference 28–28.5 in
rugby ball|rugby union|0.41-0.46||0.29||| 410–460 g; length 280–300 mm
volleyball,volleyballs|volleyball|0.26-0.28|0.21|||| circumference 65–67 cm, 260–280 g
basketball ball,size 7 basketball|basketball|0.567-0.65|0.239|||| size 7 (men): circumference 29.5 in (74.9 cm), 20–22 oz
golf ball|golf|0.04593|0.04267|||| at most 45.93 g; at least 42.67 mm across
tennis ball|tennis|0.056-0.0594|0.067|||| 56.0–59.4 g; diameter 6.54–6.86 cm
table tennis ball,ping pong ball|table tennis|0.0027|0.04|||| 40 mm, 2.7 g
badminton birdie,shuttlecock,badminton shuttlecock|badminton|0.00474-0.0055|0.068|0.07||| 16 feathers fixed in a cork base 25–28 mm across; 4.74–5.50 g; feathers 62–70 mm long, tips forming a circle 58–68 mm across
ice hockey puck,hockey puck|ice hockey|0.156-0.17|0.0762|||0.0254| vulcanized rubber, 1 in thick, 3 in across, 5.5–6 oz
field hockey ball|field hockey|0.156-0.163|0.073|||| 156–163 g; circumference 224–235 mm
lacrosse ball|lacrosse|0.14-0.149|0.0635|||| solid rubber, 5–5.25 oz
squash ball|squash|0.023-0.025|0.04|||| 39.5–40.5 mm, 23–25 g
water polo ball|water polo|0.4-0.45|0.22|||| men's size 5: circumference 68–71 cm
handball ball|handball|0.425-0.475|0.185|||| men's size 3: circumference 58–60 cm
tennis racket,tennis racquet|tennis|0.25-0.35||0.737|0.3175|| ITF: overall length at most 29 in (73.7 cm); head at most 12.5 in (31.75 cm) wide and 15.5 in (39.4 cm) long
baseball bat|baseball|0.85-1.0|0.0699|1.067|||MLB: at most 42 in long and 2.75 in across at the thickest point; solid wood
cricket bat|cricket|1.1-1.4||0.965|0.108|| at most 38 in (96.5 cm) long and 4.25 in (10.8 cm) wide; blade at most 2.64 in deep
golf club|golf|0.3-0.35||1.168||| at most 46 in (116.8 cm) long, excluding putters; driver heads at most 460 cm³
hurdle,hurdles,track hurdle|athletics|10|||1.2|1.067| men's 110 m hurdles: 1.067 m (42 in); women's 100 m: 0.838 m (33 in); men's 400 m: 0.914 m (36 in); women's 400 m: 0.762 m (30 in); hurdles are 1.18–1.20 m wide
javelin|athletics|0.8|0.03|2.65||| men's: 800 g, 2.6–2.7 m; women's: 600 g, 2.2–2.3 m
shot put|athletics|7.26|0.12|||| men's: 7.26 kg (16 lb), 110–130 mm across; women's: 4 kg
discus|athletics|2.0|0.22|||| men's: 2 kg, 219–221 mm across; women's: 1 kg
hammer throw hammer|athletics|7.26||1.215||| men's: 7.26 kg, overall length at most 121.5 cm; women's: 4 kg
bobsled,bobsleigh,four-man bobsled,4-man bobsled|bobsleigh|210-630||3.8|0.67|| four-man: at most 3.8 m long; minimum sled mass 210 kg and at most 630 kg with crew. Two-man: at most 2.7 m long; at most 390 kg with crew. Width at most 0.67 m
two-man bobsled,2-man bobsled|bobsleigh|170-390||2.7|0.67|| at most 2.7 m long, sled at least 170 kg, at most 390 kg with crew
curling stone,curling rock|curling|17.24-19.96|0.2873||||granite; circumference at most 91.44 cm; 38–44 lb
dartboard|darts||0.451|||1.73| 451 mm across; bullseye 1.73 m (5 ft 8 in) above the floor; throw line 2.37 m from the board
";

/// names | sport | length m | width m | note
const FIELDS: &str = "\
football field,american football field,nfl field,gridiron|American football|109.73|48.77|120 yd × 53⅓ yd (360 × 160 ft) including two 10-yard end zones; 100 yd between the goal lines; area 5,351 m² (1.32 acres)
soccer field,football pitch,soccer pitch,association football pitch|association football|105|68|FIFA recommends 105 × 68 m; the Laws of the Game allow 100–110 × 64–75 m for international matches (90–120 × 45–90 m otherwise)
gaelic football field,gaelic football pitch,gaelic pitch|Gaelic football and hurling|137.5|85|130–145 m × 80–90 m; H-shaped goals with a crossbar 2.5 m high
rugby field,rugby pitch,rugby union pitch|rugby union|100|70|at most 100 m between the try lines and 70 m wide; in-goal areas 6–22 m deep
tennis court|tennis|23.77|10.97|23.77 m long; 8.23 m wide for singles, 10.97 m for doubles; net 0.914 m high at the centre
basketball court,nba court|basketball|28.65|15.24|NBA: 94 × 50 ft; FIBA: 28 × 15 m; rim 10 ft (3.05 m) high
volleyball court|volleyball|18|9|18 × 9 m; net 2.43 m (men) or 2.24 m (women)
badminton court|badminton|13.4|6.1|13.4 m long; 6.1 m wide for doubles, 5.18 m for singles; net 1.524 m at the centre
ice hockey rink,hockey rink,nhl rink|ice hockey|60.96|25.91|NHL: 200 × 85 ft; IIHF: 60 × 26–30 m
baseball field,baseball diamond|baseball|27.43|27.43|90 ft (27.43 m) between bases; pitcher's rubber 60 ft 6 in from home plate; outfield fences typically 325–400 ft from home
cricket pitch|cricket|20.12|3.05|22 yd (20.12 m) between the stumps; 10 ft (3.05 m) wide; the field is an oval roughly 140–150 m across
swimming pool,olympic swimming pool,olympic pool|swimming|50|25|50 × 25 m, 10 lanes of 2.5 m, at least 2 m deep (3 m recommended); about 2,500 m³
running track,athletics track,400m track|athletics|176.91|92.52|400 m in lane 1; 8 or 9 lanes 1.22 m wide; straights 84.39 m, bend radius 36.5 m
";

/// names | sport | league | city | founded | home venue | titles | note
const TEAMS: &str = "\
dallas cowboys|American football|NFL (NFC East)|Arlington, Texas|1960|AT&T Stadium|5 Super Bowls (1971, 1977, 1992, 1993, 1995 seasons)|\"America's Team\"; the most valuable sports franchise in the world
green bay packers,packers|American football|NFL (NFC North)|Green Bay, Wisconsin|1919|Lambeau Field|13 league championships, including 4 Super Bowls (I, II, XXXI, XLV)|the only community-owned major American professional team
new england patriots|American football|NFL (AFC East)|Foxborough, Massachusetts|1960|Gillette Stadium|6 Super Bowls (2001–2018 seasons)|
pittsburgh steelers,steelers|American football|NFL (AFC North)|Pittsburgh, Pennsylvania|1933|Acrisure Stadium|6 Super Bowls|
san francisco 49ers,49ers|American football|NFL (NFC West)|Santa Clara, California|1946|Levi's Stadium|5 Super Bowls|
kansas city chiefs|American football|NFL (AFC West)|Kansas City, Missouri|1960|Arrowhead Stadium|4 Super Bowls (IV, LIV, LVII, LVIII)|
philadelphia eagles|American football|NFL (NFC East)|Philadelphia, Pennsylvania|1933|Lincoln Financial Field|2 Super Bowls (LII, LIX) and 3 earlier NFL championships|
chicago bears|American football|NFL (NFC North)|Chicago, Illinois|1920|Soldier Field|9 NFL championships, including Super Bowl XX|
miami dolphins|American football|NFL (AFC East)|Miami Gardens, Florida|1966|Hard Rock Stadium|2 Super Bowls (VII, VIII)|the only perfect season in NFL history (1972, 17–0)
new york giants|American football|NFL (NFC East)|East Rutherford, New Jersey|1925|MetLife Stadium|8 NFL championships, including 4 Super Bowls|
new york jets|American football|NFL (AFC East)|East Rutherford, New Jersey|1960|MetLife Stadium|1 Super Bowl (III)|
los angeles lakers,lakers|basketball|NBA (Western Conference)|Los Angeles, California|1947|Crypto.com Arena|17 NBA championships|founded in Minneapolis; moved to Los Angeles in 1960
boston celtics,celtics|basketball|NBA (Eastern Conference)|Boston, Massachusetts|1946|TD Garden|18 NBA championships (the most), most recently 2024|
chicago bulls|basketball|NBA (Eastern Conference)|Chicago, Illinois|1966|United Center|6 NBA championships (1991–93, 1996–98)|the Michael Jordan era
golden state warriors|basketball|NBA (Western Conference)|San Francisco, California|1946|Chase Center|7 NBA championships (1947, 1956, 1975, 2015, 2017, 2018, 2022)|founded in Philadelphia; moved to the Bay Area in 1962
cleveland cavaliers,cavaliers,cavs|basketball|NBA (Eastern Conference)|Cleveland, Ohio|1970|Rocket Arena|1 NBA championship (2016)|came back from 3–1 down in the 2016 Finals
new york knicks,knicks|basketball|NBA (Eastern Conference)|New York City|1946|Madison Square Garden|2 NBA championships (1970, 1973)|
new york yankees,yankees|baseball|MLB (American League East)|The Bronx, New York City|1901|Yankee Stadium|27 World Series titles (the most), most recently 2009|founded as the Baltimore Orioles; in New York since 1903
boston red sox,red sox|baseball|MLB (American League East)|Boston, Massachusetts|1901|Fenway Park|9 World Series titles, most recently 2018|
los angeles dodgers,dodgers|baseball|MLB (National League West)|Los Angeles, California|1883|Dodger Stadium|9 World Series titles (1955, 1959, 1963, 1965, 1981, 1988, 2020, 2024, 2025)|in Brooklyn until 1957
chicago cubs|baseball|MLB (National League Central)|Chicago, Illinois|1876|Wrigley Field|3 World Series titles (1907, 1908, 2016)|
st. louis cardinals,st louis cardinals|baseball|MLB (National League Central)|St. Louis, Missouri|1882|Busch Stadium|11 World Series titles|
montreal canadiens,canadiens,habs|ice hockey|NHL (Eastern Conference)|Montreal, Quebec|1909|Bell Centre|24 Stanley Cups (the most)|
toronto maple leafs,maple leafs|ice hockey|NHL (Eastern Conference)|Toronto, Ontario|1917|Scotiabank Arena|13 Stanley Cups, the last in 1967|
real madrid,real madrid cf|association football|La Liga|Madrid, Spain|1902|Santiago Bernabéu Stadium|15 European Cups / Champions Leagues (the most) and 36 Spanish league titles|
fc barcelona,barcelona fc,barça,barca|association football|La Liga|Barcelona, Spain|1899|Spotify Camp Nou|5 European Cups / Champions Leagues and 28 Spanish league titles|
manchester united,man united,man utd|association football|Premier League|Manchester, England|1878|Old Trafford|20 English league titles and 3 European Cups|
liverpool fc,liverpool football club|association football|Premier League|Liverpool, England|1892|Anfield|20 English league titles and 6 European Cups|
bayern munich,fc bayern munich,fc bayern|association football|Bundesliga|Munich, Germany|1900|Allianz Arena|6 European Cups and over 30 German league titles|
kentucky wildcats,kentucky wildcats men's basketball,university of kentucky basketball|basketball|NCAA Division I (Southeastern Conference)|Lexington, Kentucky|1903|Rupp Arena|8 NCAA championships (1948, 1949, 1951, 1958, 1978, 1996, 1998, 2012)|the winningest program in college basketball history
duke blue devils,duke blue devils men's basketball|basketball|NCAA Division I (Atlantic Coast Conference)|Durham, North Carolina|1906|Cameron Indoor Stadium|5 NCAA championships (1991, 1992, 2001, 2010, 2015)|
ohio state buckeyes football,ohio state buckeyes,ohio state football|American football|NCAA Division I FBS (Big Ten Conference)|Columbus, Ohio|1890|Ohio Stadium|9 claimed national championships, most recently the 2024 season|7 Heisman Trophy winners
alabama crimson tide football,alabama crimson tide|American football|NCAA Division I FBS (Southeastern Conference)|Tuscaloosa, Alabama|1892|Bryant–Denny Stadium|18 claimed national championships|
michigan wolverines football,michigan wolverines|American football|NCAA Division I FBS (Big Ten Conference)|Ann Arbor, Michigan|1879|Michigan Stadium|12 claimed national championships, most recently the 2023 season|the winningest program in college football history
";

/// names | city | opened | capacity | construction cost | tenants | note
const VENUES: &str = "\
fenway park|Boston, Massachusetts|1912-04-20|37,755 (day games)|$650,000 (1912)|Boston Red Sox (MLB)|the oldest active Major League Baseball park; its 37-ft left-field wall is the Green Monster
lambeau field|Green Bay, Wisconsin|1957-09-29|81,441|$960,000 (1957); $295 million renovation (2003)|Green Bay Packers (NFL)|the oldest continually operating NFL stadium
wembley stadium,wembley|London, England|2007-03-09|90,000|£798 million|England national football team; FA Cup final|the largest stadium in the UK; its 133 m arch is a London landmark
beijing national stadium,bird's nest,birds nest stadium|Beijing, China|2008-06-28|80,000 (91,000 for the 2008 Olympics)|US$428 million|2008 Summer Olympics and Paralympics; 2022 Winter Olympics ceremonies|steel lattice design by Herzog & de Meuron with Ai Weiwei
metlife stadium|East Rutherford, New Jersey|2010-04-10|82,500|$1.6 billion|New York Giants and New York Jets (NFL)|hosted Super Bowl XLVIII and the 2026 FIFA World Cup final
yankee stadium|The Bronx, New York City|2009-04-02|46,537|$2.3 billion|New York Yankees (MLB), New York City FC (MLS)|
wrigley field|Chicago, Illinois|1914-04-23|41,649|$250,000 (1914)|Chicago Cubs (MLB)|ivy-covered outfield walls
dodger stadium|Los Angeles, California|1962-04-10|56,000|$23 million (1962)|Los Angeles Dodgers (MLB)|the largest MLB ballpark by capacity
at&t stadium,cowboys stadium|Arlington, Texas|2009-05-27|80,000 (expandable to about 105,000)|$1.3 billion|Dallas Cowboys (NFL)|retractable roof
madison square garden,msg|New York City|1968-02-11|19,812 (basketball)|$123 million (1968)|New York Knicks (NBA), New York Rangers (NHL)|the fourth arena of that name
michigan stadium,the big house|Ann Arbor, Michigan|1927-10-01|107,601|$950,000 (1927)|Michigan Wolverines football|the largest stadium in the United States
ohio stadium,the horseshoe|Columbus, Ohio|1922-10-07|102,780|$1.3 million (1922)|Ohio State Buckeyes football|
rupp arena|Lexington, Kentucky|1976-11-27|20,545|$53 million|Kentucky Wildcats men's basketball|
rose bowl,rose bowl stadium|Pasadena, California|1922-10-28|89,702|$272,198 (1922)|UCLA Bruins football; the Rose Bowl Game|
camp nou,spotify camp nou,nou camp|Barcelona, Spain|1957-09-24|about 105,000 after renovation (99,354 before)|288 million pesetas (1957)|FC Barcelona|the largest stadium in Europe
old trafford|Manchester, England|1910-02-19|about 74,000|£60,000 (1910)|Manchester United|\"the Theatre of Dreams\"
santiago bernabéu stadium,santiago bernabeu,bernabéu,bernabeu|Madrid, Spain|1947-12-14|about 83,000|—|Real Madrid|retractable roof added in a 2019–2024 renovation
maracanã,maracana,maracanã stadium|Rio de Janeiro, Brazil|1950-06-16|78,838|—|Flamengo, Fluminense|hosted the 1950 and 2014 World Cup finals; nearly 200,000 attended the 1950 final
melbourne cricket ground,mcg|Melbourne, Australia|1853|100,024|—|Australian cricket and Australian rules football|the largest cricket ground in the world
rungrado 1st of may stadium,rungrado may day stadium|Pyongyang, North Korea|1989-05-01|114,000 (claimed)|—|North Korea national football team; mass games|the largest stadium in the world by stated capacity
";

/// athlete | Olympic medals: gold, silver, bronze | note
const MEDALS: &[(&[&str], u32, u32, u32, &str)] = &[
    (&["michael phelps", "phelps"], 23, 3, 2, "swimming, 2004–2016; the most Olympic medals and golds of any athlete"),
    (&["simone biles", "biles"], 7, 2, 2, "artistic gymnastics, 2016–2024"),
    (&["usain bolt", "bolt"], 8, 0, 0, "athletics, 2008–2016 (a 2008 relay gold was later stripped for a teammate's doping)"),
    (&["katie ledecky", "ledecky"], 9, 4, 1, "swimming, 2012–2024"),
    (&["larisa latynina", "latynina"], 9, 5, 4, "artistic gymnastics, 1956–1964"),
    (&["marit bjørgen", "marit bjorgen", "bjørgen"], 8, 4, 3, "cross-country skiing, 2002–2018; the most decorated Winter Olympian"),
    (&["ole einar bjørndalen", "ole einar bjorndalen"], 8, 4, 1, "biathlon, 1994–2014"),
    (&["bjørn dæhlie", "bjorn daehlie"], 8, 4, 0, "cross-country skiing, 1992–1998"),
    (&["carl lewis"], 9, 1, 0, "athletics, 1984–1996"),
    (&["paavo nurmi"], 9, 3, 0, "athletics, 1920–1928"),
    (&["mark spitz"], 9, 1, 1, "swimming, 1968–1972; 7 golds in 1972"),
    (&["nadia comăneci", "nadia comaneci"], 5, 3, 1, "artistic gymnastics, 1976–1980; the first perfect 10"),
    (&["jesse owens"], 4, 0, 0, "athletics, 1936"),
    (&["shaun white"], 3, 0, 0, "snowboard halfpipe: 2006, 2010, 2018"),
    (&["kjetil andré aamodt", "kjetil andre aamodt", "aamodt"], 4, 2, 2, "alpine skiing (Norway), 1992–2006; the most Olympic medals of any alpine skier"),
    (&["aksel lund svindal", "svindal"], 2, 1, 1, "alpine skiing (Norway), 2010–2018"),
];

fn rows(d: &'static str) -> Vec<Vec<&'static str>> {
    d.lines().map(|l| l.split('|').collect()).collect()
}

fn equipment() -> &'static Vec<Vec<&'static str>> {
    static E: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    E.get_or_init(|| rows(EQUIPMENT))
}
fn fields() -> &'static Vec<Vec<&'static str>> {
    static E: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    E.get_or_init(|| rows(FIELDS))
}
fn teams() -> &'static Vec<Vec<&'static str>> {
    static E: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    E.get_or_init(|| rows(TEAMS))
}
fn venues() -> &'static Vec<Vec<&'static str>> {
    static E: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    E.get_or_init(|| rows(VENUES))
}

fn hit(r: &[&str], n: &str) -> bool {
    r[0].split(',').any(|x| x == n)
}

fn title(s: &str) -> String {
    s.split(' ').map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join(" ")
        .replace("Nfl", "NFL").replace("Nba", "NBA").replace("Nhl", "NHL").replace("Fc", "FC").replace("At&t", "AT&T").replace("Mcg", "MCG").replace("49Ers", "49ers").replace("St. Louis", "St. Louis")
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn mass_text(s: &str) -> String {
    let parts: Vec<f64> = s.split('-').filter_map(|x| x.parse().ok()).collect();
    let one = |kg: f64| if kg < 1.0 { format!("{} g", fmt_float((kg * 1000.0 * 100.0).round() / 100.0, 5)) } else { format!("{} kg", fmt_float(kg, 4)) };
    let lb = |kg: f64| if kg < 1.0 { format!("{} oz", fmt_float((kg * 35.274 * 100.0).round() / 100.0, 4)) } else { format!("{} lb", fmt_float((kg * 2.20462 * 100.0).round() / 100.0, 4)) };
    match parts.as_slice() {
        [a] => format!("{} ({})", one(*a), lb(*a)),
        [a, b] => format!("{} to {} ({} to {})", one(*a), one(*b), lb(*a), lb(*b)),
        _ => String::new(),
    }
}

fn len_text(m: f64) -> String {
    if m < 1.0 {
        format!("{} cm ({} in)", fmt_float((m * 1000.0).round() / 10.0, 5), fmt_float((m / 0.0254 * 100.0).round() / 100.0, 4))
    } else {
        format!("{} m ({} ft)", fmt_float((m * 100.0).round() / 100.0, 5), fmt_float((m / 0.3048 * 10.0).round() / 10.0, 5))
    }
}

fn equip_value(r: &[&str], key: &str) -> Option<String> {
    let f = |i: usize| r[i].parse::<f64>().ok();
    match key {
        "mass" | "weight" | "heavy" => (!r[2].is_empty()).then(|| mass_text(r[2])),
        "diameter" | "size" | "width across" => f(3).map(len_text),
        "circumference" => f(3).map(|d| len_text(d * std::f64::consts::PI)),
        "length" | "long" => f(4).map(len_text),
        "width" | "wide" | "head width" => f(5).map(len_text),
        "height" | "tall" | "high" => f(6).map(len_text),
        _ => None,
    }
}

fn equip_card(r: &[&str]) -> Vec<Vec<Content>> {
    let mut v = vec![vec![text("sport"), text(r[1])]];
    for (k, lab) in [("mass", "mass"), ("diameter", "diameter"), ("length", "length"), ("width", "width"), ("height", "height")] {
        if let Some(x) = equip_value(r, k) {
            v.push(vec![text(lab), text(x)]);
        }
    }
    v.push(vec![text("rules"), text(r[7].trim())]);
    v
}

fn field_card(r: &[&str]) -> Vec<Vec<Content>> {
    let (l, w) = (r[2].parse::<f64>().unwrap_or(0.0), r[3].parse::<f64>().unwrap_or(0.0));
    vec![
        vec![text("sport"), text(r[1])],
        vec![text("length"), text(len_text(l))],
        vec![text("width"), text(len_text(w))],
        vec![text("area"), text(format!("{} m² ({} ft², {} acres)", fmt_float(l * w, 5), fmt_float((l * w * 10.7639).round(), 6), fmt_float((l * w / 4046.86 * 100.0).round() / 100.0, 3)))],
        vec![text("rules"), text(r[4])],
    ]
}

fn medals(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:how many (?:olympic )?medals (?:has|did|does) (.+?) (?:won|win|have)|(?:number of )?(?:olympic )?medals (?:won by|of|for) (.+)|(.+?)(?:'s)? (?:number of )?(?:olympic )?(?:gold )?medals(?: count)?|(?:olympic )?medals (?:has|did) (.+?) (?:won|win))$").captures(l)?;
    let who = m.get(1).or(m.get(2)).or(m.get(3)).or(m.get(4))?.as_str().trim();
    let (names, g, s, b, note) = MEDALS.iter().find(|x| x.0.contains(&who))?;
    let name = title(names[0]);
    let total = g + s + b;
    let main = if l.contains("gold") { format!("{g} gold medals") } else { format!("{total} Olympic medals ({g} gold, {s} silver, {b} bronze)") };
    Some(Answer::new("medals", vec![
        input(format!("{name} | Olympic medals")),
        primary(pod("Result", "Olympic medals", text(main))),
        pod("Table", "Medals", table(Some(vec!["gold", "silver", "bronze", "total"]), vec![vec![text(g.to_string()), text(s.to_string()), text(b.to_string()), text(total.to_string())]])),
        pod("Note", "Note", text(*note)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").replace('’', "'");
    let l = re!(r"^(?:what is |what are |what's |show |the )").replace(&l, "").to_string();
    if let Some(a) = medals(&l) {
        return Some(a);
    }
    // "how much did it cost to build metlife stadium", "construction cost of lambeau field"
    if let Some(m) = re!(r"^(?:how much did it cost to build|(?:construction )?cost (?:to build|of building|of)|construction cost of|capacity of|seating capacity of|how many (?:people|seats|fans)(?: does| can)? (?:fit in )?|when (?:was|did) (?:the )?)(?:the )?(.+?)(?: (?:open|built|hold|seat))?$").captures(&l) {
        if let Some(r) = venues().iter().find(|r| hit(r, m[1].trim())) {
            let (lab, v) = if l.contains("cost") { ("construction cost", r[4]) } else if l.starts_with("when") { ("opened", r[2]) } else { ("capacity", r[3]) };
            return Some(Answer::new("venue", vec![input(format!("{} | {lab}", title(r[0].split(',').next()?))), primary(pod("Result", lab, text(v)))]));
        }
    }
    // properties of equipment and fields: "mass of a bowling ball vs. cricket ball", "head width of tennis racket", "bobsled length"
    let prop = re!(r"^(?:(mass|weight|diameter|circumference|size|length|width|head width|height|dimensions|area) (?:of )?(?:an? |the )?(.+)|(.+?) (mass|weight|diameter|circumference|size|length|width|height|dimensions|area)|how (heavy|long|wide|tall|high|big) (?:is|are) (?:an? |the )?(.+))$").captures(&l);
    if let Some(m) = prop {
        let (key, what) = if let Some(k) = m.get(1) { (k.as_str(), m[2].to_string()) } else if let Some(k) = m.get(4) { (k.as_str(), m[3].to_string()) } else { (m.get(5)?.as_str(), m[6].to_string()) };
        let names: Vec<String> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*(?:an? |the )?").split(&what).map(|s| s.trim().to_string()).collect();
        let eq: Option<Vec<&Vec<&str>>> = names.iter().map(|n| equipment().iter().find(|r| hit(r, n))).collect();
        if let Some(eq) = eq {
            let vals: Option<Vec<String>> = eq.iter().map(|r| equip_value(r, key)).collect();
            if let Some(vals) = vals {
                let main = if eq.len() == 1 { vals[0].clone() } else { eq.iter().zip(&vals).map(|(r, v)| format!("{}: {v}", r[0].split(',').next().unwrap_or(""))).collect::<Vec<_>>().join(" | ") };
                let mut pods = vec![input(format!("{} | {key}", eq.iter().map(|r| r[0].split(',').next().unwrap_or("")).collect::<Vec<_>>().join(", "))), primary(pod("Result", key, text(main)))];
                for r in &eq {
                    pods.push(pod("Properties", r[0].split(',').next().unwrap_or(""), table(None, equip_card(r))));
                }
                return Some(Answer::new("sports equipment", pods));
            }
        }
        let fl: Option<Vec<&Vec<&str>>> = names.iter().map(|n| fields().iter().find(|r| hit(r, n))).collect();
        if let Some(fl) = fl {
            let val = |r: &Vec<&str>| -> String {
                let (a, b) = (r[2].parse::<f64>().unwrap_or(0.0), r[3].parse::<f64>().unwrap_or(0.0));
                match key { "length" => len_text(a), "width" => len_text(b), "area" => format!("{} m²", fmt_float(a * b, 5)), _ => format!("{} × {}", len_text(a), len_text(b)) }
            };
            let main = if fl.len() == 1 { val(fl[0]) } else { fl.iter().map(|r| format!("{}: {}", r[0].split(',').next().unwrap_or(""), val(r))).collect::<Vec<_>>().join(" | ") };
            return Some(Answer::new("playing area", vec![input(format!("{} | {key}", fl.iter().map(|r| r[0].split(',').next().unwrap_or("")).collect::<Vec<_>>().join(", "))), primary(pod("Result", key, text(main)))]));
        }
    }
    // bare names and comparisons (a bare sport name means the sport, not its ball)
    if matches!(l.as_str(), "baseball" | "baseballs" | "softball" | "softballs" | "volleyball" | "volleyballs" | "discus" | "javelin") {
        return None;
    }
    let names: Vec<&str> = re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+)\s*").split(&l).map(|s| s.trim()).collect();
    if let Some(eq) = names.iter().map(|n| equipment().iter().find(|r| hit(r, n))).collect::<Option<Vec<_>>>() {
        let mut pods = vec![input(format!("{} | sports equipment", eq.iter().map(|r| r[0].split(',').next().unwrap_or("")).collect::<Vec<_>>().join(", ")))];
        pods.push(primary(pod("Result", "Rules", text(eq.iter().map(|r| r[7].trim()).collect::<Vec<_>>().join(" | ")))));
        for r in &eq {
            pods.push(pod("Properties", r[0].split(',').next().unwrap_or(""), table(None, equip_card(r))));
        }
        return Some(Answer::new("sports equipment", pods));
    }
    if let Some(fl) = names.iter().map(|n| fields().iter().find(|r| hit(r, n))).collect::<Option<Vec<_>>>() {
        let mut pods = vec![input(format!("{} | playing area", fl.iter().map(|r| r[0].split(',').next().unwrap_or("")).collect::<Vec<_>>().join(", ")))];
        pods.push(primary(pod("Result", "Dimensions", text(fl.iter().map(|r| format!("{}: {}", r[0].split(',').next().unwrap_or(""), r[4])).collect::<Vec<_>>().join(" | ")))));
        for r in &fl {
            pods.push(pod("Properties", r[0].split(',').next().unwrap_or(""), table(None, field_card(r))));
        }
        return Some(Answer::new("playing area", pods));
    }
    if names.len() == 1 {
        let n = names[0];
        if let Some(r) = teams().iter().find(|r| hit(r, n) && (n.contains(' ') || n.len() > 6)) {
            return Some(Answer::new("team", vec![
                input(format!("{} | sports team", title(r[0].split(',').next()?))),
                primary(pod("Result", "Summary", text(format!("{} team ({}) based in {}, founded {}", r[1], r[2], r[3], r[4])))),
                pod("Properties", "Basic information", table(None, vec![
                    vec![text("sport"), text(r[1])],
                    vec![text("league"), text(r[2])],
                    vec![text("location"), text(r[3])],
                    vec![text("founded"), text(r[4])],
                    vec![text("home venue"), text(r[5])],
                    vec![text("championships"), text(r[6])],
                    vec![text("note"), text(r[7])],
                ].into_iter().filter(|row| !matches!(&row[1], Content::Text { text: t } if t.is_empty())).collect())),
            ]));
        }
        if let Some(r) = venues().iter().find(|r| hit(r, n)) {
            let opened = chrono::NaiveDate::parse_from_str(r[2], "%Y-%m-%d").map(|d| d.format("%B %-d, %Y").to_string()).unwrap_or_else(|_| r[2].to_string());
            return Some(Answer::new("venue", vec![
                input(format!("{} | stadium", title(r[0].split(',').next()?))),
                primary(pod("Result", "Summary", text(format!("stadium in {}, opened {}; capacity {}", r[1], opened, r[3])))),
                pod("Properties", "Basic information", table(None, vec![
                    vec![text("location"), text(r[1])],
                    vec![text("opened"), text(opened)],
                    vec![text("capacity"), text(r[3])],
                    vec![text("construction cost"), text(r[4])],
                    vec![text("tenants"), text(r[5])],
                    vec![text("note"), text(r[6])],
                ].into_iter().filter(|row| !matches!(&row[1], Content::Text { text: t } if t.is_empty() || t == "—")).collect())),
            ]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_parse() {
        for r in equipment() {
            assert_eq!(r.len(), 8, "{r:?}");
        }
        for r in fields() {
            assert_eq!(r.len(), 5, "{r:?}");
            assert!(r[2].parse::<f64>().is_ok() && r[3].parse::<f64>().is_ok());
        }
        for r in teams() {
            assert_eq!(r.len(), 8, "{r:?}");
        }
        for r in venues() {
            assert_eq!(r.len(), 7, "{r:?}");
        }
    }
}
