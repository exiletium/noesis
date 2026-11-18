//! Personal names: origin and meaning of common given names and surnames,
//! and the most popular US baby names.
//!
//! "name William", "What does the name Cassandra mean?", "name Jones",
//! "5 most popular girl's names", "origin of the surname Busch".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// name | usage | origin | meaning
const GIVEN: &str = "\
william|male|Germanic (Old High German Willahelm, brought to England by the Normans)|resolute protector (wil 'will, desire' + helm 'helmet, protection')
liam|male|Irish short form of William|resolute protector
daniel|male|Hebrew (Daniyyel)|God is my judge
danielle|female|French feminine form of Daniel (Hebrew)|God is my judge
jessie|unisex|Scottish pet form of Janet; also short for Jessica (female) or Jesse (male)|Jesse: Hebrew 'gift'; Janet: from John, 'God is gracious'
jamie|unisex|Scottish diminutive of James|James: from Hebrew Ya'aqov, 'holder of the heel, supplanter'
cassandra|female|Greek (Kassandra)|possibly 'shining upon man' (kazō 'to excel, shine' + anēr 'man'); the Trojan prophetess no one believed
bella|female|Italian, from Latin bellus; also short for Isabella|beautiful
isabella|female|Spanish and Italian form of Elizabeth (Hebrew)|my God is an oath
elizabeth|female|Hebrew (Elisheva)|my God is an oath
wolfram|male|Germanic|wolf + raven (wolf + hraban)
olivia|female|Latin oliva, popularized by Shakespeare's Twelfth Night|olive
emma|female|Germanic, short form of names with ermen|whole, universal
amelia|female|Germanic, from names with amal|work
charlotte|female|French feminine diminutive of Charles|free man
mia|female|Scandinavian and German short form of Maria|Maria: from Hebrew Miryam, meaning uncertain ('bitter', 'beloved' and 'drop of the sea' have been proposed)
sophia|female|Greek|wisdom
sofia|female|Spanish, Italian and Slavic form of Sophia (Greek)|wisdom
ava|female|uncertain; possibly Germanic (Aveza) or a form of Eve (Hebrew 'life')|uncertain
evelyn|unisex|English surname, from the Norman name Aveline|uncertain
noah|male|Hebrew (Noach)|rest, comfort
oliver|male|Old French Olivier, associated with Latin oliva|olive tree
james|male|Hebrew Ya'aqov via Late Latin Iacomus|holder of the heel, supplanter
theodore|male|Greek (Theodōros)|gift of God
henry|male|Germanic (Heimirich)|ruler of the home
mateo|male|Spanish form of Matthew (Hebrew)|gift of God
elijah|male|Hebrew (Eliyyahu)|my God is Yahweh
lucas|male|Latin form of Greek Loukas|man from Lucania
benjamin|male|Hebrew (Binyamin)|son of the right hand (or son of the south)
john|male|Hebrew (Yochanan) via Greek and Latin|God is gracious
jack|male|English pet form of John|God is gracious
michael|male|Hebrew (Mikha'el)|who is like God?
david|male|Hebrew (Dawid)|beloved
joseph|male|Hebrew (Yosef)|he will add
thomas|male|Aramaic (Ta'oma)|twin
robert|male|Germanic (Hrodebert)|bright fame
richard|male|Germanic (Ricohard)|brave ruler
charles|male|Germanic (Karl)|free man
george|male|Greek (Geōrgios)|farmer, earth-worker
edward|male|Old English (Eadweard)|rich guardian
alexander|male|Greek (Alexandros)|defender of men
matthew|male|Hebrew (Mattityahu)|gift of Yahweh
andrew|male|Greek (Andreas)|manly
peter|male|Greek (Petros)|stone, rock
paul|male|Latin (Paulus)|small, humble
mark|male|Latin (Marcus)|probably from the god Mars
christopher|male|Greek (Christophoros)|bearing Christ
anthony|male|Latin (the Roman family name Antonius)|unknown, probably of Etruscan origin
stephen|male|Greek (Stephanos)|crown, wreath
samuel|male|Hebrew (Shemu'el)|God has heard (or name of God)
ethan|male|Hebrew (Etan)|firm, enduring
jacob|male|Hebrew (Ya'aqov)|holder of the heel, supplanter
arthur|male|Brythonic, of uncertain origin|possibly 'bear' (Welsh arth)
harry|male|medieval English form of Henry|ruler of the home
mary|female|Hebrew (Miryam) via Greek and Latin|uncertain ('bitter', 'beloved' and 'drop of the sea' have been proposed)
maria|female|Latin form of Mary (Hebrew Miryam)|uncertain
anna|female|Greek and Latin form of Hannah (Hebrew)|favour, grace
hannah|female|Hebrew (Channah)|favour, grace
sarah|female|Hebrew|princess, lady
emily|female|Latin (Aemilia, a Roman family name)|rival (aemulus)
grace|female|English virtue name|grace
chloe|female|Greek|green shoot
zoe|female|Greek|life
lily|female|English flower name|the lily flower, a symbol of purity
jessica|female|Hebrew (Yiskah), first used in this form by Shakespeare|to behold
jennifer|female|Cornish form of Guinevere (Welsh Gwenhwyfar)|fair, white + smooth
katherine|female|Greek (Aikaterinē), later linked to katharos|uncertain; traditionally 'pure'
margaret|female|Greek (margaritēs)|pearl
victoria|female|Latin|victory
alice|female|Germanic (Adalheidis) via Old French|noble type
laura|female|Latin (laurus)|laurel
abigail|female|Hebrew (Avigayil)|my father is joy
madison|female|English surname|son of Maud (or Matthew)
harper|unisex|English occupational surname|harp player
ella|female|Germanic|all, other
scarlett|female|English surname for a seller or dyer of scarlet cloth|scarlet
luna|female|Latin|moon
aria|female|Italian (an operatic solo); also Persian and Hebrew|air, song
nicole|female|French feminine of Nicholas (Greek)|victory of the people
nicholas|male|Greek (Nikolaos)|victory of the people
rachel|female|Hebrew|ewe
rebecca|female|Hebrew (Rivqah)|possibly 'to tie, bind'
natalie|female|Late Latin natale domini|Christmas day
julia|female|Latin (the Roman family name Julius)|possibly 'downy-bearded, youthful'
";

/// surname | origin | meaning
const SURNAMES: &str = "\
jones|Welsh and English patronymic|son of John (Hebrew 'God is gracious')
smith|English occupational|metalworker, blacksmith
johnson|English patronymic|son of John
williams|Welsh and English patronymic|son of William
brown|English and Scottish nickname|brown-haired or brown-skinned
adams|English patronymic|son of Adam (Hebrew 'man', related to adamah 'earth')
busch|German topographic|someone who lived by a bush or thicket
miller|English occupational|miller
davis|Welsh patronymic|son of David
wilson|English patronymic|son of Will (William)
anderson|Scottish and English patronymic|son of Andrew
taylor|English occupational|tailor
müller|German occupational|miller
garcia|Spanish, probably of Basque origin|uncertain; possibly 'young' or 'bear'
kim|Korean|gold, metal (金)
nguyen|Vietnamese|the Nguyễn dynasty (about 40% of Vietnamese people bear it)
wang|Chinese|king (王)
li|Chinese|plum tree (李)
";

/// US Social Security Administration, babies born in 2024
const TOP_GIRLS: &[&str] = &["Olivia", "Emma", "Amelia", "Charlotte", "Mia", "Sophia", "Isabella", "Evelyn", "Ava", "Sofia"];
const TOP_BOYS: &[&str] = &["Liam", "Noah", "Oliver", "Theodore", "James", "Henry", "Mateo", "Elijah", "Lucas", "William"];

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn given(n: &str) -> Option<[&'static str; 4]> {
    GIVEN.lines().map(|l| l.split('|').collect::<Vec<_>>()).find(|f| f[0] == n).map(|f| [f[0], f[1], f[2], f[3]])
}

fn surname(n: &str) -> Option<[&'static str; 3]> {
    let n = n.replace("mueller", "müller");
    SURNAMES.lines().map(|l| l.split('|').collect::<Vec<_>>()).find(|f| f[0] == n).map(|f| [f[0], f[1], f[2]])
}

fn popular(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the )?(?:top )?(\d+|ten|five)? ?(?:(?:most popular|most common|top|popular) )?(?:(girls?'?s?|boys?'?s?|female|male|baby) )?(?:baby )?(?:first )?names(?: in the us| in the united states| in america)?(?: (?:in )?2024)?$").captures(l)?;
    if !re!(r"\b(?:popular|top|common)\b").is_match(l) {
        return None;
    }
    let k = match m.get(1).map(|x| x.as_str()) {
        Some("ten") => 10,
        Some("five") => 5,
        Some(x) => x.parse().ok()?,
        None => 10,
    };
    if k == 0 || k > 10 {
        return None;
    }
    let who = m.get(2).map(|x| x.as_str()).unwrap_or("baby");
    let (title, rows): (&str, Vec<Vec<Content>>) = if who.starts_with("girl") || who == "female" {
        ("girls' names", TOP_GIRLS[..k].iter().enumerate().map(|(i, n)| vec![text((i + 1).to_string()), text(*n)]).collect())
    } else if who.starts_with("boy") || who == "male" {
        ("boys' names", TOP_BOYS[..k].iter().enumerate().map(|(i, n)| vec![text((i + 1).to_string()), text(*n)]).collect())
    } else {
        ("baby names", (0..k).map(|i| vec![text((i + 1).to_string()), text(TOP_GIRLS[i]), text(TOP_BOYS[i])]).collect())
    };
    let result = if title == "baby names" {
        format!("girls: {} | boys: {}", TOP_GIRLS[..k].join(", "), TOP_BOYS[..k].join(", "))
    } else if title.starts_with("girls") {
        TOP_GIRLS[..k].join(" | ")
    } else {
        TOP_BOYS[..k].join(" | ")
    };
    let head = if title == "baby names" { Some(vec!["rank", "girls", "boys"]) } else { Some(vec!["rank", "name"]) };
    Some(Answer::new(format!("most popular {title}"), vec![
        pod("Input", "Input interpretation", text(format!("{k} most popular {title} (United States, babies born in 2024)"))),
        primary(pod("Result", "Result", text(result))),
        pod("Table", "Ranking", table(head, rows)),
        pod("Source", "Source", text("US Social Security Administration, names of babies born in 2024")),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim().replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    if let Some(a) = popular(&l) {
        return Some(a);
    }
    let m = re!(r"^(?:(?:the )?(?:first |given |last |family )?(name|surname) ([a-zü]+)|([a-zü]+) (?:first |given |last )?(name|surname)(?: meaning| origin)?|what does (?:the )?(?:first |last )?(name|surname) ([a-zü]+) mean|(?:meaning|origin|etymology) of (?:the )?(?:first |given |last |family )?(name|surname) ([a-zü]+)|what is the (?:meaning|origin) of (?:the )?(?:first |last )?(name|surname) ([a-zü]+))$").captures(&l)?;
    let (kind, n) = [(1, 2), (4, 3), (5, 6), (7, 8), (9, 10)].iter().find_map(|(k, n)| Some((m.get(*k)?.as_str(), m.get(*n)?.as_str())))?;
    let last = kind == "surname" || l.contains("last name") || l.contains("family name");
    if !last {
        if let Some(f) = given(n) {
            return Some(Answer::new(format!("{} (given name)", cap(n)), vec![
                pod("Input", "Input interpretation", text(format!("{} (given name)", cap(n)))),
                primary(pod("Result", "Meaning", text(f[3]))),
                pod("Table", "Basic information", table(None, vec![
                    vec![text("usage"), text(f[1])],
                    vec![text("origin"), text(f[2])],
                    vec![text("meaning"), text(f[3])],
                ])),
                pod("Popularity", "US popularity (2024)", text(
                    TOP_GIRLS.iter().position(|x| x.eq_ignore_ascii_case(n)).map(|i| format!("#{} for girls", i + 1))
                        .or_else(|| TOP_BOYS.iter().position(|x| x.eq_ignore_ascii_case(n)).map(|i| format!("#{} for boys", i + 1)))
                        .unwrap_or_else(|| "not in the top 10".into()))),
            ]));
        }
    }
    let f = surname(n)?;
    Some(Answer::new(format!("{} (surname)", cap(f[0])), vec![
        pod("Input", "Input interpretation", text(format!("{} (surname)", cap(f[0])))),
        primary(pod("Result", "Meaning", text(f[2]))),
        pod("Table", "Basic information", table(None, vec![vec![text("type"), text(f[1])], vec![text("meaning"), text(f[2])]])),
    ]))
}
