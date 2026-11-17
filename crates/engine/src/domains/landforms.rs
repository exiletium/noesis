//! Physical geography: mountains, volcanoes, rivers, lakes, waterfalls,
//! deserts, islands, oceans and seas, ocean trenches, caves, canyons and
//! forests. Single features, comparisons ("Tigris, Euphrates"), one property
//! ("length Amazon / Nile"), sums ("surface area Lake Erie + Lake Huron"),
//! ranked lists ("longest rivers"), major volcanic eruptions and distances
//! from a feature to a city or another feature.

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

/// names | kind | location | lat,lon | properties | note
///
/// elev, prom, height, drop, width, depth, avgdepth: metres · len, coast:
/// kilometres · area, basin: km² · vol: km³ · flow: m³/s · pop: people.
/// Text properties: range, ascent, erupt (last eruption), arc (volcanic
/// arc), source, mouth, river, deepest (deepest point), type.
const DATA: &str = "\
mount everest,everest,mt everest,sagarmatha,chomolungma|mountain|Nepal–China border|27.9881,86.925|elev=8848.86;prom=8848.86;range=Himalayas (Mahalangur Himal);ascent=29 May 1953 (Edmund Hillary and Tenzing Norgay)|the highest mountain above sea level
k2,mount godwin-austen,chhogori|mountain|Pakistan–China border|35.8825,76.5133|elev=8611;prom=4017;range=Karakoram;ascent=31 July 1954 (Achille Compagnoni and Lino Lacedelli)|second-highest mountain on Earth
kangchenjunga,kanchenjunga|mountain|Nepal–India border|27.7025,88.1475|elev=8586;prom=3922;range=Himalayas;ascent=25 May 1955 (Joe Brown and George Band)|third-highest mountain
lhotse|mountain|Nepal–China border|27.9617,86.9333|elev=8516;prom=610;range=Himalayas;ascent=18 May 1956|
makalu|mountain|Nepal–China border|27.8897,87.0889|elev=8485;prom=2378;range=Himalayas;ascent=15 May 1955|
cho oyu|mountain|Nepal–China border|28.0942,86.6608|elev=8188;prom=2340;range=Himalayas;ascent=19 October 1954|
nanga parbat|mountain|Pakistan|35.2375,74.5892|elev=8126;prom=4608;range=Himalayas;ascent=3 July 1953 (Hermann Buhl)|
annapurna,annapurna i|mountain|Nepal|28.5961,83.8203|elev=8091;prom=2984;range=Himalayas;ascent=3 June 1950 (Maurice Herzog and Louis Lachenal)|the first eight-thousander climbed
aconcagua|mountain|Argentina|-32.6532,-70.0109|elev=6961;prom=6961;range=Andes;ascent=1897 (Matthias Zurbriggen)|highest mountain outside Asia
chimborazo|volcano|Ecuador|-1.4692,-78.8175|elev=6263;prom=4123;range=Andes;erupt=about 550 AD|its summit is the point on Earth farthest from the centre of the Earth
ojos del salado|volcano|Chile–Argentina border|-27.1092,-68.5414|elev=6893;prom=3688;range=Andes;erupt=about 750 AD|the highest volcano on Earth
cotopaxi|volcano|Ecuador|-0.6809,-78.4378|elev=5897;prom=2400;range=Andes;erupt=2022–2023|
denali,mount mckinley,mt mckinley|mountain|Alaska, United States|63.0692,-151.007|elev=6190;prom=6144;range=Alaska Range;ascent=7 June 1913 (Hudson Stuck, Harry Karstens and others)|highest mountain in North America
mount logan|mountain|Yukon, Canada|60.5672,-140.405|elev=5959;prom=5250;range=Saint Elias Mountains;ascent=23 June 1925|highest mountain in Canada
pico de orizaba,citlaltepetl,citlaltépetl|volcano|Mexico|19.0303,-97.2681|elev=5636;prom=4922;erupt=1846|highest mountain in Mexico
popocatepetl,popocatépetl|volcano|Mexico|19.0225,-98.6278|elev=5426;prom=1615;erupt=ongoing (since 2005)|
mount kilimanjaro,kilimanjaro,kibo|volcano|Tanzania|-3.0674,37.3556|elev=5895;prom=5885;ascent=6 October 1889 (Hans Meyer and Ludwig Purtscheller);erupt=dormant (last activity about 200,000 years ago)|highest mountain in Africa and the highest free-standing mountain
mount kenya|mountain|Kenya|-0.1521,37.3084|elev=5199;prom=3825;ascent=13 September 1899 (Halford Mackinder)|
mount elbrus,elbrus|volcano|Russia|43.355,42.4392|elev=5642;prom=4741;range=Caucasus;ascent=1874 (west summit);erupt=about 50 AD|highest mountain in Europe
mount ararat,ararat|volcano|Turkey|39.7019,44.2983|elev=5137;prom=3611;erupt=Holocene|
vinson massif,mount vinson|mountain|Antarctica|-78.5254,-85.6171|elev=4892;prom=4892;range=Ellsworth Mountains;ascent=18 December 1966|highest mountain in Antarctica
puncak jaya,carstensz pyramid|mountain|Papua, Indonesia|-4.0833,137.1833|elev=4884;prom=4884;range=Sudirman Range;ascent=1962 (Heinrich Harrer)|highest mountain in Oceania
mount kosciuszko,kosciuszko|mountain|New South Wales, Australia|-36.4559,148.2636|elev=2228;prom=2228;range=Snowy Mountains|highest mountain on mainland Australia
mont blanc,monte bianco|mountain|France–Italy border|45.8326,6.8652|elev=4806;prom=4696;range=Alps;ascent=8 August 1786 (Jacques Balmat and Michel-Gabriel Paccard)|highest mountain in the Alps; the summit height varies with the ice cap
matterhorn,monte cervino|mountain|Switzerland–Italy border|45.9763,7.6586|elev=4478;prom=1040;range=Alps (Pennine Alps);ascent=14 July 1865 (Edward Whymper's party)|
mount olympus|mountain|Greece|40.0859,22.3583|elev=2918;prom=2353|highest mountain in Greece; home of the gods in Greek mythology
ben nevis|mountain|Scotland, United Kingdom|56.7969,-5.0036|elev=1345;prom=1345;range=Grampian Mountains|highest mountain in the British Isles
table mountain|mountain|Cape Town, South Africa|-33.9628,18.4098|elev=1085;prom=1055|flat-topped mountain overlooking Cape Town
aoraki,mount cook,aoraki / mount cook|mountain|South Island, New Zealand|-43.595,170.1418|elev=3724;prom=3724;range=Southern Alps|highest mountain in New Zealand
mount fuji,mt fuji,fujisan,fuji-san|volcano|Honshu, Japan|35.3606,138.7274|elev=3776;prom=3776;erupt=1707–1708 (Hōei eruption)|highest mountain in Japan
mount whitney|mountain|California, United States|36.5785,-118.2923|elev=4421;prom=3072;range=Sierra Nevada;ascent=18 August 1873|highest summit in the contiguous United States
mount rainier,mt rainier,tahoma|volcano|Washington, United States|46.8523,-121.7603|elev=4392;prom=4026;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=about 1450 (minor activity in the 1800s)|the most glaciated peak in the contiguous United States
mount hood,mt hood,wy'east|volcano|Oregon, United States|45.3735,-121.6959|elev=3429;prom=2349;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=1866 (minor)|highest mountain in Oregon
mount st. helens,mount st helens,mt st helens,mt. st. helens|volcano|Washington, United States|46.1914,-122.1956|elev=2549;prom=1404;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=2004–2008|its eruption of 18 May 1980 removed the top 400 m of the mountain and killed 57 people
mount shasta|volcano|California, United States|41.4092,-122.1949|elev=4322;prom=2994;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=1786|
mount baker|volcano|Washington, United States|48.7768,-121.8145|elev=3286;prom=2686;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=1880|
lassen peak|volcano|California, United States|40.4882,-121.5049|elev=3189;prom=1594;range=Cascade Range;arc=Cascade Volcanic Arc;erupt=1914–1917|
mount timpanogos,mt timpanogos,timpanogos|mountain|Utah, United States|40.3908,-111.6458|elev=3582;prom=1782;range=Wasatch Range|
pikes peak|mountain|Colorado, United States|38.8405,-105.0442|elev=4302;prom=1680;range=Front Range|
mount washington|mountain|New Hampshire, United States|44.2706,-71.3033|elev=1917;prom=1882;range=Presidential Range|known for extreme weather; a wind gust of 372 km/h was recorded in 1934
mount mitchell|mountain|North Carolina, United States|35.7650,-82.2652|elev=2037;prom=1856;range=Black Mountains (Appalachians)|highest point east of the Mississippi River
mauna kea|volcano|Hawaii, United States|19.8207,-155.4681|elev=4207;prom=4207;erupt=about 2460 BC|about 10,210 m tall measured from its base on the ocean floor
mauna loa|volcano|Hawaii, United States|19.4721,-155.5922|elev=4169;prom=2169;erupt=27 November – 13 December 2022|the largest active volcano on Earth by volume
kilauea,kīlauea|volcano|Hawaii, United States|19.4069,-155.2834|elev=1247;erupt=ongoing (frequent eruptions since 1983)|one of the most active volcanoes on Earth
mount etna,etna|volcano|Sicily, Italy|37.751,14.9934|elev=3357;prom=3357;erupt=ongoing|the highest active volcano in Europe; the height varies with eruptions
mount vesuvius,vesuvius|volcano|Campania, Italy|40.8214,14.4261|elev=1281;erupt=1944|its eruption of 79 AD buried Pompeii and Herculaneum
stromboli|volcano|Aeolian Islands, Italy|38.7938,15.2111|elev=924;erupt=ongoing (nearly continuous for over 2,000 years)|
mount teide,teide|volcano|Tenerife, Spain|28.2723,-16.6425|elev=3715;prom=3715;erupt=1909|highest point in Spain
eyjafjallajokull,eyjafjallajökull|volcano|Iceland|63.6314,-19.6083|elev=1651;erupt=2010|its 2010 eruption grounded air travel across Europe for six days
mount erebus|volcano|Ross Island, Antarctica|-77.5294,167.1531|elev=3794;prom=3794;erupt=ongoing|southernmost active volcano, with a persistent lava lake
mount pinatubo,pinatubo|volcano|Luzon, Philippines|15.1429,120.35|elev=1486;erupt=1991 (VEI 6)|the 1991 eruption cooled the global climate by about 0.5 °C
krakatoa,krakatau,anak krakatau|volcano|Sunda Strait, Indonesia|-6.102,105.423|elev=157;erupt=ongoing (Anak Krakatau); great eruption 26–27 August 1883 (VEI 6)|the 1883 eruption and its tsunamis killed about 36,000 people and were heard 4,800 km away; Anak Krakatau rose from the caldera in 1927
mount tambora,tambora|volcano|Sumbawa, Indonesia|-8.25,118.0|elev=2850;erupt=1967 (minor); great eruption 10 April 1815 (VEI 7)|the 1815 eruption, the largest in recorded history, killed about 71,000 people and caused the 'year without a summer' (1816); the mountain was about 4,300 m high before it
mount pelee,mount pelée,montagne pelée|volcano|Martinique, France|14.8094,-61.1653|elev=1397;erupt=1929–1932|its eruption of 8 May 1902 destroyed Saint-Pierre and killed about 28,000 people
nevado del ruiz|volcano|Colombia|4.892,-75.3236|elev=5321;erupt=ongoing (since 2012)|lahars from its 1985 eruption buried Armero and killed about 23,000 people
irruputuncu|volcano|Chile–Bolivia border|-20.73,-68.56|elev=5163;arc=Central Volcanic Zone of the Andes;erupt=1995|stratovolcano with fumaroles at the summit
yellowstone caldera,yellowstone supervolcano|volcano|Wyoming, United States|44.4,-110.7|elev=2805;erupt=about 640,000 years ago (Lava Creek eruption, VEI 8); last lava flow about 70,000 years ago|a supervolcano; the caldera is about 55 × 72 km
nile,nile river,river nile,the nile|river|Northeast Africa (11 countries)|31.4675,30.3653|len=6650;flow=2830;basin=3349000;source=Lake Victoria (White Nile) and Lake Tana (Blue Nile);mouth=Mediterranean Sea|traditionally the longest river on Earth
amazon river,the amazon,river amazon|river|South America (Peru, Colombia, Brazil)|-0.1,-50.0|len=6400;flow=209000;basin=7000000;source=Andes of Peru;mouth=Atlantic Ocean|the largest river by discharge, about a fifth of all river water reaching the oceans; some measurements (up to 6,992 km) make it longer than the Nile
yangtze,yangtze river,chang jiang|river|China|31.3833,121.9833|len=6300;flow=30166;basin=1808500;source=Tibetan Plateau;mouth=East China Sea|longest river in Asia
mississippi river,the mississippi|river|United States|29.15,-89.25|len=3730;flow=16792;basin=2981076;source=Lake Itasca, Minnesota;mouth=Gulf of Mexico|the Mississippi–Missouri system is 6,275 km long, the fourth-longest in the world
missouri river|river|United States|38.8139,-90.1197|len=3767;flow=2470;basin=1371017;source=Rocky Mountains, Montana;mouth=Mississippi River near St. Louis|the longest river in North America
yellow river,huang he|river|China|37.7667,119.25|len=5464;flow=2571;basin=752546;source=Bayan Har Mountains;mouth=Bohai Sea|
yenisei,yenisei river|river|Russia, Mongolia|71.8333,82.7167|len=3487;flow=19600;basin=2580000;source=Tuva (Great Yenisei and Little Yenisei);mouth=Kara Sea|the Yenisei–Angara–Selenga system is about 5,539 km long
ob river,ob|river|Russia|66.75,69.5|len=3650;flow=12475;basin=2990000;source=Altai Mountains;mouth=Gulf of Ob (Kara Sea)|the Ob–Irtysh system is about 5,410 km long
congo river,river congo,zaire river|river|Central Africa|-6.0,12.4|len=4700;flow=41200;basin=4014500;mouth=Atlantic Ocean|the deepest river in the world (over 220 m)
mekong,mekong river|river|Southeast Asia (6 countries)|9.5,106.5|len=4350;flow=16000;basin=795000;source=Tibetan Plateau;mouth=South China Sea|
niger river|river|West Africa|4.3,6.1|len=4180;flow=5589;basin=2117700;mouth=Gulf of Guinea|
lena,lena river|river|Russia|72.4,126.6|len=4400;flow=17100;basin=2490000;mouth=Laptev Sea|
amur,amur river|river|Russia–China border|52.95,141.1|len=2824;flow=11400;basin=1855000;mouth=Strait of Tartary|with the Argun the system is about 4,444 km long
mackenzie river|river|Northwest Territories, Canada|69.35,-134.0|len=1738;flow=9910;basin=1805200;source=Great Slave Lake;mouth=Beaufort Sea|the Mackenzie system is about 4,241 km long
volga,volga river|river|Russia|45.8,47.9|len=3531;flow=8060;basin=1360000;source=Valdai Hills;mouth=Caspian Sea|the longest river in Europe
danube,danube river|river|Central and Eastern Europe (10 countries)|45.2167,29.75|len=2850;flow=6500;basin=801463;source=Black Forest, Germany;mouth=Black Sea|
rhine,rhine river,river rhine|river|Switzerland, Germany, Netherlands and others|51.9,4.1|len=1233;flow=2330;basin=185000;source=Swiss Alps;mouth=North Sea|
thames,river thames,thames river|river|England, United Kingdom|51.5,0.6|len=346;flow=65.8;basin=12935;source=Cotswolds;mouth=North Sea (Thames Estuary)|
seine,seine river|river|France|49.4347,0.2167|len=777;flow=560;basin=78650;mouth=English Channel|
loire,loire river|river|France|47.28,-2.17|len=1006;flow=840;basin=117054;mouth=Bay of Biscay|the longest river in France
elbe,elbe river|river|Czech Republic, Germany|53.92,8.72|len=1094;flow=870;basin=148268;mouth=North Sea|
tiber,tiber river|river|Italy|41.7406,12.2333|len=406;flow=267;basin=17375;mouth=Tyrrhenian Sea|the river of Rome
dnieper,dnieper river|river|Russia, Belarus, Ukraine|46.5,32.25|len=2201;flow=1670;basin=504000;mouth=Black Sea|
tigris,tigris river|river|Turkey, Syria, Iraq|30.0,48.0|len=1850;flow=1014;basin=375000;source=Taurus Mountains (Lake Hazar);mouth=Shatt al-Arab (Persian Gulf)|with the Euphrates it bounds Mesopotamia
euphrates,euphrates river|river|Turkey, Syria, Iraq|31.0,47.4|len=2800;flow=356;basin=500000;source=Armenian Highlands (Kara Su and Murat);mouth=Shatt al-Arab (Persian Gulf)|the longest river of Western Asia
jordan river,river jordan|river|Israel, Jordan, Syria, West Bank|31.76,35.56|len=251;flow=13;basin=18000;mouth=Dead Sea|
ganges,ganga,ganges river|river|India, Bangladesh|22.0,90.0|len=2525;flow=16650;basin=1080000;source=Gangotri Glacier;mouth=Bay of Bengal|sacred river of Hinduism
indus,indus river|river|China, India, Pakistan|23.99,67.43|len=3180;flow=6600;basin=1165000;source=Tibetan Plateau;mouth=Arabian Sea|
zambezi,zambezi river|river|Southern Africa (6 countries)|-18.8,36.3|len=2574;flow=3400;basin=1390000;mouth=Indian Ocean|flows over Victoria Falls
orinoco,orinoco river|river|Venezuela, Colombia|8.6,-60.8|len=2140;flow=33000;basin=880000;mouth=Atlantic Ocean|
parana river,paraná river|river|Brazil, Paraguay, Argentina|-34.0,-58.4|len=4880;flow=17290;basin=2582672;mouth=Río de la Plata|
colorado river|river|United States, Mexico|31.8,-114.8|len=2330;flow=637;basin=637137;source=Rocky Mountains (La Poudre Pass Lake);mouth=Gulf of California|carved the Grand Canyon
columbia river|river|Canada, United States|46.25,-124.05|len=2000;flow=7500;basin=668000;mouth=Pacific Ocean|
hudson river|river|New York, United States|40.7,-74.02|len=507;flow=620;basin=36260;source=Lake Tear of the Clouds;mouth=Upper New York Bay|
ohio river|river|United States|36.99,-89.13|len=1579;flow=7960;basin=490601;source=Pittsburgh (Allegheny and Monongahela);mouth=Mississippi River at Cairo, Illinois|
rio grande|river|United States, Mexico|25.96,-97.15|len=3051;flow=160;basin=870000;mouth=Gulf of Mexico|
st. lawrence river,st lawrence river,saint lawrence river|river|Canada, United States|49.5,-67.0|len=1197;flow=16800;basin=1344200;source=Lake Ontario;mouth=Gulf of Saint Lawrence|
murray river,river murray|river|Australia|-35.56,138.88|len=2508;flow=767;basin=1061469;mouth=Southern Ocean (Lake Alexandrina)|the Murray–Darling system is about 3,672 km long
caspian sea,the caspian sea|lake|Kazakhstan, Russia, Azerbaijan, Iran, Turkmenistan|41.67,50.67|area=371000;depth=1025;vol=78200;elev=-28|the largest lake on Earth by area and volume; brackish
lake superior|lake|United States, Canada|47.7,-87.5|area=82100;depth=406;vol=12100;elev=183|the largest freshwater lake by area
lake victoria|lake|Uganda, Kenya, Tanzania|-1.0,33.0|area=59947;depth=81;vol=2424;elev=1134|the largest lake in Africa and the largest tropical lake
lake huron|lake|United States, Canada|44.8,-82.4|area=59600;depth=229;vol=3540;elev=176|
lake michigan|lake|United States|44.0,-87.0|area=58000;depth=281;vol=4900;elev=176|the largest lake entirely within one country
lake tanganyika|lake|Tanzania, DR Congo, Burundi, Zambia|-6.5,29.5|area=32600;depth=1470;vol=18900;elev=773|the longest freshwater lake (673 km) and the second-deepest
lake baikal,baikal|lake|Siberia, Russia|53.5,108.0|area=31722;depth=1642;vol=23615;elev=456|the deepest and oldest lake on Earth, holding about 22 % of the world's fresh surface water
great bear lake|lake|Northwest Territories, Canada|65.92,-120.75|area=31153;depth=446;vol=2236;elev=156|
lake malawi,lake nyasa|lake|Malawi, Mozambique, Tanzania|-12.0,34.5|area=29600;depth=706;vol=8400;elev=468|
great slave lake|lake|Northwest Territories, Canada|61.67,-114.0|area=27200;depth=614;vol=1580;elev=156|the deepest lake in North America
lake erie|lake|United States, Canada|42.2,-81.2|area=25700;depth=64;vol=484;elev=174|the shallowest of the Great Lakes
lake winnipeg|lake|Manitoba, Canada|52.12,-97.25|area=24514;depth=36;vol=284;elev=217|
lake ontario|lake|United States, Canada|43.7,-77.9|area=18960;depth=244;vol=1640;elev=74|
lake ladoga,ladoga|lake|Russia|60.83,31.5|area=17700;depth=230;vol=908;elev=5|the largest lake in Europe
lake titicaca,titicaca|lake|Peru, Bolivia|-15.75,-69.42|area=8372;depth=281;vol=893;elev=3812|the largest lake in South America and the highest navigable lake
great salt lake|lake|Utah, United States|41.17,-112.58|area=4400;depth=10;elev=1280|a salt lake; its area varies from about 2,500 to 8,500 km² with the water level
lake tahoe,tahoe|lake|California–Nevada, United States|39.09,-120.04|area=496;depth=501;vol=150.7;elev=1897|
crater lake|lake|Oregon, United States|42.94,-122.1|area=53.2;depth=594;vol=18.7;elev=1883|the deepest lake in the United States, in the caldera of Mount Mazama
loch ness|lake|Scottish Highlands, United Kingdom|57.3,-4.45|area=56;depth=230;vol=7.4;elev=16|holds more fresh water than all the lakes of England and Wales together; home of the legendary monster
dead sea|lake|Israel, Jordan, West Bank|31.5,35.5|area=605;depth=298;vol=114;elev=-430|its shore is the lowest land on Earth; salinity about 34 %
lake geneva,lac léman,lac leman|lake|Switzerland, France|46.45,6.53|area=580;depth=310;vol=89;elev=372|
lake garda,lago di garda|lake|Italy|45.63,10.67|area=370;depth=346;vol=49;elev=65|the largest lake in Italy
lake nasser|lake|Egypt, Sudan|22.42,31.75|area=5250;depth=180;vol=132;elev=183|reservoir behind the Aswan High Dam
lake mead|lake|Nevada–Arizona, United States|36.25,-114.39|area=640;depth=162;vol=35.2;elev=372|reservoir behind the Hoover Dam
lake okeechobee|lake|Florida, United States|26.95,-80.8|area=1900;depth=4;vol=5.2;elev=4|
angel falls,kerepakupai meru|waterfall|Bolívar, Venezuela|5.9675,-62.5356|height=979;drop=807;river=Kerepacupai Merú (Churún River)|the highest uninterrupted waterfall on Earth
tugela falls|waterfall|KwaZulu-Natal, South Africa|-28.7522,28.8944|height=948;drop=411;river=Tugela River|
victoria falls,mosi-oa-tunya|waterfall|Zambia–Zimbabwe border|-17.9243,25.8572|height=108;width=1708;flow=1088;river=Zambezi|the largest sheet of falling water on Earth
iguazu falls,iguacu falls,iguaçu falls,iguassu falls|waterfall|Argentina–Brazil border|-25.6953,-54.4367|height=82;width=2700;flow=1746;river=Iguazu River|about 275 separate falls along 2.7 km
niagara falls|waterfall|New York, United States – Ontario, Canada|43.0799,-79.0747|height=51;width=1039;flow=2400;river=Niagara River|three falls: Horseshoe Falls (51 m high, 820 m wide), American Falls and Bridal Veil Falls
horseshoe falls|waterfall|Ontario, Canada – New York, United States|43.0771,-79.0757|height=51;width=820;flow=2270;river=Niagara River|the largest of the three Niagara Falls
yosemite falls|waterfall|Yosemite National Park, California, United States|37.7566,-119.5969|height=739;drop=436;river=Yosemite Creek|the tallest waterfall in Yosemite; it often dries up in late summer
colonial creek falls|waterfall|North Cascades National Park, Washington, United States|48.6878,-121.1078|height=788;river=Colonial Creek|one of the tallest waterfalls in the United States
multnomah falls|waterfall|Oregon, United States|45.5762,-122.1158|height=189;drop=165;river=Multnomah Creek|
bambarakanda falls|waterfall|Badulla District, Sri Lanka|6.7726,80.8283|height=263;river=Kuda Oya|the tallest waterfall in Sri Lanka
kaieteur falls|waterfall|Guyana|5.175,-59.48|height=226;width=113;flow=663;river=Potaro River|one of the most powerful single-drop waterfalls
sutherland falls|waterfall|Fiordland, New Zealand|-44.8,167.73|height=580;river=Arthur River|
jog falls|waterfall|Karnataka, India|14.2294,74.8125|height=253;river=Sharavathi River|
rhine falls,rheinfall|waterfall|Schaffhausen, Switzerland|47.6779,8.6153|height=23;width=150;flow=600;river=Rhine|the largest plain waterfall in Europe by flow
gullfoss|waterfall|Iceland|64.3271,-20.1199|height=32;river=Hvítá|
skogafoss,skógafoss|waterfall|Iceland|63.5321,-19.5114|height=60;width=25;river=Skógá|
dettifoss|waterfall|Iceland|65.8147,-16.3846|height=44;width=100;flow=193;river=Jökulsá á Fjöllum|the most powerful waterfall in Europe
antarctic desert|desert|Antarctica|-82.0,0.0|area=14200000;type=polar|the largest desert on Earth
arctic desert|desert|Arctic|80.0,0.0|area=13900000;type=polar|
sahara,sahara desert,the sahara|desert|North Africa (11 countries)|23.4,13.0|area=9200000;type=subtropical (hot)|the largest hot desert on Earth
arabian desert|desert|Arabian Peninsula|22.0,48.0|area=2330000;type=subtropical (hot)|
gobi desert,gobi,the gobi|desert|Mongolia, China|42.59,103.43|area=1295000;type=cold winter (rain shadow)|the largest desert in Asia
kalahari desert,kalahari|desert|Botswana, Namibia, South Africa|-23.0,22.0|area=900000;type=semi-arid|
patagonian desert|desert|Argentina, Chile|-45.0,-69.0|area=673000;type=cold winter|
syrian desert|desert|Syria, Jordan, Iraq, Saudi Arabia|32.0,40.0|area=500000;type=subtropical (hot)|
great victoria desert|desert|Western Australia, South Australia|-29.0,127.0|area=348750;type=subtropical (hot)|the largest desert in Australia
taklamakan desert,taklamakan,taklimakan|desert|Xinjiang, China|38.9,82.2|area=337000;type=cold winter|
karakum desert,kara kum,karakum|desert|Turkmenistan|40.0,60.0|area=350000;type=cold winter|
great sandy desert|desert|Western Australia|-20.5,123.5|area=284993;type=subtropical (hot)|
sonoran desert|desert|Arizona, California, Mexico|32.0,-112.0|area=260000;type=subtropical (hot)|
thar desert,great indian desert|desert|India, Pakistan|27.0,71.0|area=200000;type=subtropical (hot)|
simpson desert|desert|Australia|-25.0,137.0|area=176500;type=subtropical (hot)|
gibson desert|desert|Western Australia|-23.5,124.0|area=156000;type=subtropical (hot)|
mojave desert,mojave|desert|California, Nevada, Utah, Arizona|35.0,-115.5|area=124000;type=rain shadow|contains Death Valley
atacama desert,atacama|desert|Chile|-24.5,-69.25|area=105000;type=coastal (cold current)|the driest non-polar desert on Earth
namib desert,namib|desert|Namibia, Angola, South Africa|-24.75,15.28|area=81000;type=coastal (cold current)|possibly the oldest desert, at least 55 million years
death valley|desert|California, United States|36.24,-116.82|elev=-86;area=7800|lowest point in North America (Badwater Basin, −86 m) and the hottest recorded air temperature, 56.7 °C on 10 July 1913
new guinea|island|Papua New Guinea and Indonesia|-5.0,141.0|area=785753;pop=15000000;elev=4884|the second-largest island on Earth
borneo|island|Indonesia, Malaysia, Brunei|0.5,114.0|area=743330;pop=23000000;elev=4095|the third-largest island
baffin island|island|Nunavut, Canada|68.0,-70.0|area=507451;pop=13000;elev=2147|the largest island of Canada
sumatra|island|Indonesia|0.0,102.0|area=473481;pop=60000000;elev=3805|
honshu|island|Japan|36.0,138.0|area=227960;pop=104000000;elev=3776|the main island of Japan
great britain|island|United Kingdom|54.0,-2.0|area=209331;pop=65000000;elev=1345|the largest island in Europe
victoria island|island|Canada|71.0,-110.0|area=217291;pop=2200;elev=655|
ellesmere island|island|Nunavut, Canada|79.8,-78.0|area=196236;pop=150;elev=2616|
sulawesi,celebes|island|Indonesia|-2.0,121.0|area=180681;pop=20000000;elev=3478|
south island|island|New Zealand|-43.5,171.0|area=150437;pop=1200000;elev=3724|
hispaniola|island|Haiti and the Dominican Republic|19.0,-71.0|area=76192;pop=23000000;elev=3098|the second-largest island of the Caribbean, shared by Haiti and the Dominican Republic; highest point Pico Duarte
sicily|island|Italy|37.6,14.0|area=25711;pop=4800000;elev=3357|the largest island in the Mediterranean
sardinia|island|Italy|40.0,9.0|area=24090;pop=1600000;elev=1834|
corsica|island|France|42.15,9.1|area=8680;pop=350000;elev=2706|
crete|island|Greece|35.2,24.9|area=8450;pop=620000;elev=2456|
galapagos islands,galápagos islands,galapagos|island|Ecuador|-0.83,-91.13|area=8010;pop=25000;elev=1707|volcanic archipelago about 1,000 km west of Ecuador; Charles Darwin visited in 1835
bali|island|Indonesia|-8.34,115.09|area=5780;pop=4300000;elev=3031|
long island|island|New York, United States|40.8,-73.3|area=3629;pop=8000000;elev=122|
easter island,rapa nui|island|Chile|-27.12,-109.35|area=163.6;pop=7750;elev=507|known for its moai statues
martha's vineyard,marthas vineyard|island|Massachusetts, United States|41.4,-70.62|area=231;pop=20600|
nantucket,nantucket island|island|Massachusetts, United States|41.28,-70.1|area=124;pop=14255|
maro reef|reef|Northwestern Hawaiian Islands, United States|25.42,-170.59|area=1934|a largely submerged coral atoll in the Papahānaumokuākea Marine National Monument; its reef area is about 1,934 km²
great barrier reef|reef|Queensland, Australia|-18.29,147.7|area=344400;len=2300|the largest coral reef system on Earth
pacific ocean,the pacific,pacific|ocean|between Asia and Australia and the Americas|0.0,-160.0|area=165250000;vol=710000000;avgdepth=4280;depth=10935;deepest=Challenger Deep, Mariana Trench;coast=135663|the largest and deepest ocean, about 46 % of Earth's water surface
atlantic ocean,the atlantic,atlantic|ocean|between the Americas and Europe and Africa|0.0,-30.0|area=106460000;vol=310410900;avgdepth=3646;depth=8376;deepest=Milwaukee Deep, Puerto Rico Trench;coast=111866|the second-largest ocean
indian ocean|ocean|between Africa, Asia and Australia|-20.0,80.0|area=70560000;vol=264000000;avgdepth=3741;depth=7192;deepest=Sunda (Java) Trench;coast=66526|the warmest ocean
southern ocean,antarctic ocean|ocean|around Antarctica, south of 60° S|-65.0,0.0|area=21960000;vol=71800000;avgdepth=3270;depth=7432;deepest=Factorian Deep, South Sandwich Trench;coast=17968|recognized by the International Hydrographic Organization in 2000
arctic ocean|ocean|around the North Pole|85.0,0.0|area=14060000;vol=18750000;avgdepth=1205;depth=5550;deepest=Molloy Deep, Fram Strait;coast=45389|the smallest and shallowest ocean
mediterranean sea,the mediterranean,mediterranean|sea|between Europe, Africa and Asia|35.0,18.0|area=2500000;vol=3750000;avgdepth=1500;depth=5267;deepest=Calypso Deep|
caribbean sea|sea|Caribbean|15.0,-75.0|area=2754000;vol=6860000;avgdepth=2200;depth=7686;deepest=Cayman Trough|
south china sea|sea|Southeast Asia|12.0,113.0|area=3500000;avgdepth=1212;depth=5016|
gulf of mexico|sea|United States, Mexico, Cuba|25.0,-90.0|area=1550000;vol=2434000;avgdepth=1615;depth=4384;deepest=Sigsbee Deep|
red sea|sea|between Africa and Arabia|20.0,38.0|area=438000;vol=233000;avgdepth=490;depth=3040|
black sea|sea|between Europe and Asia|43.0,35.0|area=436400;vol=547000;avgdepth=1253;depth=2212|
north sea|sea|between Great Britain and Scandinavia|56.0,3.0|area=570000;vol=54000;avgdepth=95;depth=700|
baltic sea|sea|Northern Europe|58.0,20.0|area=377000;vol=21700;avgdepth=55;depth=459|
hudson bay|sea|Canada|60.0,-85.0|area=1230000;avgdepth=100;depth=270|
bay of bengal|sea|India, Bangladesh, Myanmar, Sri Lanka|15.0,88.0|area=2172000;avgdepth=2600;depth=4694|
arabian sea|sea|between India and Arabia|15.0,65.0|area=3862000;avgdepth=2734;depth=4652|
sea of japan,east sea|sea|between Japan and Korea|40.0,135.0|area=978000;vol=1713000;avgdepth=1752;depth=3742|
bering sea|sea|between Russia and Alaska|58.0,-178.0|area=2000000;avgdepth=1547;depth=4097|
mariana trench,marianas trench|trench|western Pacific Ocean, east of the Mariana Islands|11.35,142.2|depth=10935;len=2550;wkm=69;deepest=Challenger Deep|the deepest place in the oceans; first reached in 1960 by Jacques Piccard and Don Walsh in the Trieste
tonga trench|trench|southwestern Pacific Ocean|-23.25,-174.73|depth=10882;len=860;deepest=Horizon Deep|the second-deepest trench
kermadec trench|trench|southwestern Pacific Ocean, northeast of New Zealand|-31.9,-177.3|depth=10047;len=1000;deepest=Scholl Deep|
philippine trench|trench|western Pacific Ocean, east of the Philippines|10.4,126.7|depth=10540;len=1320;wkm=30;deepest=Galathea Depth|
kuril-kamchatka trench,kuril trench|trench|northwestern Pacific Ocean|44.0,150.0|depth=9604;len=2200|
puerto rico trench|trench|Atlantic Ocean, north of Puerto Rico|19.8,-66.5|depth=8376;len=800;deepest=Milwaukee Deep|the deepest point of the Atlantic
peru-chile trench,atacama trench|trench|eastern Pacific Ocean, off South America|-23.4,-71.3|depth=8065;len=5900;deepest=Richards Deep|
south sandwich trench|trench|South Atlantic and Southern Ocean|-55.5,-26.0|depth=8265;len=965;deepest=Meteor Deep|
sunda trench,java trench|trench|Indian Ocean, south of Java and Sumatra|-10.3,110.0|depth=7192;len=3200|the deepest point of the Indian Ocean
mammoth cave|cave|Kentucky, United States|37.187,-86.1|len=686|the longest known cave system on Earth
eisriesenwelt|cave|Werfen, Salzburg, Austria|47.5031,13.1897|len=42;elev=1641|the largest ice cave in the world
son doong cave,hang son doong|cave|Quảng Bình, Vietnam|17.4566,106.2875|len=9;height=200|the largest known cave passage by volume
veryovkina cave|cave|Abkhazia, Georgia|43.4183,40.3561|depth=2212|the deepest known cave
krubera cave,voronya cave|cave|Abkhazia, Georgia|43.4092,40.3622|depth=2197|
carlsbad caverns|cave|New Mexico, United States|32.1479,-104.5567|len=50;depth=489|
howe caverns|cave|New York, United States|42.6976,-74.3994|len=2.4;depth=48|show cave about 60 m below the surface, opened to visitors in 1842
waitomo glowworm caves,waitomo caves|cave|New Zealand|-38.2608,175.1036|len=0.25|lit by glowworms (Arachnocampa luminosa)
lascaux,lascaux cave|cave|Dordogne, France|45.0539,1.1683|len=0.235|Paleolithic cave paintings about 17,000 years old
blue grotto,grotta azzurra|cave|Capri, Italy|40.5606,14.2056|len=0.06|sea cave lit blue by sunlight passing under the water
frasassi caves,grotte di frasassi|cave|Marche, Italy|43.4,12.97|len=30|
castellana caves,grotte di castellana|cave|Apulia, Italy|40.875,17.15|len=3.3;depth=122|
grotta gigante|cave|Friuli-Venezia Giulia, Italy|45.7092,13.7639|depth=107;height=98|one of the largest single cave chambers open to tourists
neptune's grotto,grotta di nettuno|cave|Sardinia, Italy|40.5636,8.1636|len=4|
grand canyon|canyon|Arizona, United States|36.1,-112.1|len=446;wkm=29;depth=1857;river=Colorado River|carved over about 5 to 6 million years
fish river canyon|canyon|Namibia|-27.6,17.6|len=160;wkm=27;depth=550;river=Fish River|the largest canyon in Africa
copper canyon,barranca del cobre|canyon|Chihuahua, Mexico|27.5,-107.75|depth=1870|
yarlung tsangpo grand canyon|canyon|Tibet, China|29.6,95.0|len=504;depth=6009;river=Yarlung Tsangpo (Brahmaputra)|the deepest canyon on Earth
amazon rainforest,amazon jungle|forest|Brazil, Peru, Colombia and six other countries|-3.0,-60.0|area=5500000|the largest tropical rainforest
sherwood forest|forest|Nottinghamshire, England, United Kingdom|53.204,-1.072|area=4.23|ancient royal forest of the Robin Hood legends; the national nature reserve covers 423 ha and includes the Major Oak
black forest,schwarzwald|forest|Baden-Württemberg, Germany|48.3,8.15|area=6009;elev=1493|wooded mountain range
bialowieza forest,białowieża forest|forest|Poland, Belarus|52.73,23.85|area=1417|one of the last primeval forests of Europe, home of the European bison
aletsch glacier,great aletsch glacier|glacier|Bernese Alps, Switzerland|46.5,8.03|len=23;area=81.7|the largest glacier in the Alps
lambert glacier|glacier|East Antarctica|-73.0,70.0|len=400;wkm=100|the largest glacier on Earth
";

struct Feature {
    names: Vec<&'static str>,
    kind: &'static str,
    place: &'static str,
    lat: f64,
    lon: f64,
    props: Vec<(&'static str, &'static str)>,
    note: &'static str,
}

fn data() -> &'static [Feature] {
    static D: OnceLock<Vec<Feature>> = OnceLock::new();
    D.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let p: Vec<&str> = l.split('|').collect();
                let (lat, lon) = p.get(3)?.split_once(',')?;
                Some(Feature {
                    names: p[0].split(',').map(str::trim).collect(),
                    kind: p[1],
                    place: p[2],
                    lat: lat.parse().ok()?,
                    lon: lon.parse().ok()?,
                    props: p.get(4)?.split(';').filter_map(|kv| kv.split_once('=')).collect(),
                    note: p.get(5).copied().unwrap_or(""),
                })
            })
            .collect()
    })
}

/// "mt hood" → "mount hood"; "the nile" → "nile"
fn norm(s: &str) -> String {
    let s = s.trim().to_lowercase().replace('’', "'");
    let s = re!(r"^(?:mt\.?|mnt\.?)\s+").replace(&s, "mount ").to_string();
    let s = s.replace(" volcano", "").replace(" mountain", "");
    s.trim().to_string()
}

fn find(name: &str) -> Option<&'static Feature> {
    let n = norm(name);
    let alts = [n.clone(), n.trim_start_matches("the ").to_string(), n.trim_end_matches(" cave").to_string()];
    alts.iter().find_map(|a| data().iter().find(|f| f.names.iter().any(|x| x == a)))
}

/// In a list or with a property, "Amazon" may mean the river and
/// "Erie" the lake.
fn find_loose(name: &str) -> Option<&'static Feature> {
    find(name).or_else(|| {
        let n = norm(name);
        [format!("mount {n}"), format!("{n} river"), format!("lake {n}"), format!("{n} desert"), format!("{n} ocean"), format!("{n} falls")].iter().find_map(|a| data().iter().find(|f| f.names.iter().any(|x| x == a)))
    })
}

impl Feature {
    fn get(&self, k: &str) -> Option<&'static str> {
        self.props.iter().find(|(a, _)| *a == k).map(|(_, v)| *v)
    }
    fn num(&self, k: &str) -> Option<f64> {
        self.get(k)?.parse().ok()
    }
    fn title(&self) -> String {
        title(self.names[0])
    }
}

/// Title case that keeps small words and "K2" readable.
fn title(s: &str) -> String {
    s.split(' ')
        .enumerate()
        .map(|(i, w)| {
            if i > 0 && matches!(w, "of" | "the" | "del" | "de" | "di" | "and") {
                w.to_string()
            } else {
                let mut c = w.chars();
                c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A stored value: kept as written, with thousands separators.
fn n(v: f64) -> String {
    group(fmt_float(v, 10))
}

/// A converted value: rounded to a sensible precision.
fn c(v: f64) -> String {
    let r = if v.abs() >= 1000.0 { v.round() } else if v.abs() >= 10.0 { (v * 10.0).round() / 10.0 } else { (v * 100.0).round() / 100.0 };
    group(fmt_float(r, 10))
}

fn group(s: String) -> String {
    let (sign, body) = match s.strip_prefix('-') {
        Some(b) => ("−", b.to_string()),
        None => ("", s),
    };
    let (int, frac) = match body.split_once('.') {
        Some((a, b)) => (a.to_string(), format!(".{b}")),
        None => (body.clone(), String::new()),
    };
    if int.len() <= 4 || !int.chars().all(|ch| ch.is_ascii_digit()) {
        return format!("{sign}{int}{frac}");
    }
    let mut out = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    format!("{sign}{out}{frac}")
}

const LABELS: &[(&str, &str)] = &[
    ("elev", "elevation"),
    ("prom", "prominence"),
    ("height", "height"),
    ("drop", "tallest single drop"),
    ("len", "length"),
    ("width", "width"),
    ("wkm", "width"),
    ("area", "area"),
    ("depth", "maximum depth"),
    ("avgdepth", "average depth"),
    ("vol", "volume"),
    ("flow", "average discharge"),
    ("basin", "drainage basin"),
    ("coast", "coastline"),
    ("pop", "population"),
    ("range", "mountain range"),
    ("arc", "volcanic arc"),
    ("erupt", "last eruption"),
    ("ascent", "first ascent"),
    ("source", "source"),
    ("mouth", "mouth"),
    ("river", "river"),
    ("deepest", "deepest point"),
    ("type", "desert type"),
];

fn value(f: &Feature, k: &str) -> Option<String> {
    let v = f.get(k)?;
    let x: Option<f64> = v.parse().ok();
    Some(match (k, x) {
        ("elev", Some(m)) if f.kind == "island" => format!("{} m ({} ft) highest point", n(m), c(m * 3.280_84)),
        ("elev" | "prom" | "height" | "drop" | "depth" | "avgdepth" | "width", Some(m)) => format!("{} m ({} ft)", n(m), c(m * 3.280_84)),
        ("wkm", Some(km)) => format!("{} km ({} mi)", n(km), c(km * 0.621_371)),
        ("len" | "coast", Some(km)) => format!("{} km ({} mi)", n(km), c(km * 0.621_371)),
        ("area" | "basin", Some(a)) if a < 10.0 => format!("{} km² ({} acres)", n(a), c(a * 247.105)),
        ("area" | "basin", Some(a)) => format!("{} km² ({} mi²)", n(a), c(a * 0.386_102)),
        ("vol", Some(v)) => format!("{} km³ ({} mi³)", n(v), c(v * 0.239_913)),
        ("flow", Some(q)) => format!("{} m³/s ({} ft³/s)", n(q), c(q * 35.3147)),
        ("pop", Some(p)) => format!("about {} people", n(p)),
        _ => v.to_string(),
    })
}

fn rows(f: &Feature) -> Vec<Vec<Content>> {
    let mut r = vec![vec![text("type"), text(f.kind)], vec![text("location"), text(f.place)]];
    r.extend(LABELS.iter().filter_map(|(k, l)| value(f, k).map(|v| vec![text(*l), text(v)])));
    r.push(vec![text("coordinates"), text(format!("{:.2}° {}, {:.2}° {}", f.lat.abs(), if f.lat >= 0.0 { "N" } else { "S" }, f.lon.abs(), if f.lon >= 0.0 { "E" } else { "W" }))]);
    r
}

fn card(f: &Feature) -> Answer {
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} ({})", f.title(), f.kind))), primary(pod("Properties", "Basic properties", table(None, rows(f))))];
    if !f.note.is_empty() {
        pods.push(pod("Note", "Note", text(f.note)));
    }
    Answer::new(f.title(), pods)
}

fn compare(fs: &[&'static Feature]) -> Answer {
    let keys: Vec<(&str, &str)> = LABELS.iter().copied().filter(|(k, _)| fs.iter().filter(|f| f.get(k).is_some()).count() >= 1).collect();
    let names: Vec<String> = fs.iter().map(|f| f.title()).collect();
    let mut head = vec![""];
    head.extend(names.iter().map(String::as_str));
    let mut t: Vec<Vec<Content>> = vec![{
        let mut r = vec![text("type")];
        r.extend(fs.iter().map(|f| text(f.kind)));
        r
    }, {
        let mut r = vec![text("location")];
        r.extend(fs.iter().map(|f| text(f.place)));
        r
    }];
    t.extend(keys.iter().map(|(k, l)| {
        let mut r = vec![text(*l)];
        r.extend(fs.iter().map(|f| text(value(f, k).unwrap_or_else(|| "—".into()))));
        r
    }));
    Answer::new("comparison", vec![pod("Input", "Input interpretation", text(names.join(" | "))), primary(pod("Result", "Comparison", table(Some(head), t)))])
}

/// The property key a phrase names.
fn prop_key(p: &str, f: &Feature) -> Option<&'static str> {
    Some(match p.trim() {
        "elevation" | "altitude" | "summit elevation" => if f.kind == "island" || f.get("elev").is_none() && f.get("height").is_some() { if f.get("elev").is_some() { "elev" } else { "height" } } else { "elev" },
        "height" | "how tall is" | "how high is" => if f.get("height").is_some() { "height" } else if f.get("elev").is_some() { "elev" } else { "depth" },
        "prominence" | "topographic prominence" => "prom",
        "length" | "how long is" => "len",
        "width" | "how wide is" => if f.get("wkm").is_some() { "wkm" } else { "width" },
        "area" | "surface area" | "size" => "area",
        "depth" | "max depth" | "maximum depth" | "how deep is" => "depth",
        "lowest point" | "deepest point" => if f.get("deepest").is_some() { "deepest" } else { "depth" },
        "average depth" | "mean depth" => "avgdepth",
        "volume" => "vol",
        "discharge" | "flow" | "flow rate" | "average discharge" => "flow",
        "drainage basin" | "basin" | "basin area" | "catchment area" => "basin",
        "coastline" | "coastline length" | "coast" => "coast",
        "population" => "pop",
        "last eruption" | "eruptions" | "last eruption of" => "erupt",
        "first ascent" => "ascent",
        "volcanic arc" | "volcanic arc of" => "arc",
        "source" => "source",
        "mouth" => "mouth",
        "mountain range" | "range" => "range",
        _ => return None,
    })
}

const PROPS: &str = r"elevation|altitude|summit elevation|height|prominence|topographic prominence|length|width|surface area|area|size|max(?:imum)? depth|depth|lowest point|deepest point|average depth|mean depth|volume|discharge|flow rate|flow|drainage basin|basin area|catchment area|basin|coastline length|coastline|population|last eruption|first ascent|volcanic arc|source|mouth|mountain range";

fn label(k: &str) -> &'static str {
    LABELS.iter().find(|(a, _)| *a == k).map(|(_, b)| *b).unwrap_or("value")
}

fn split_items(s: &str) -> Vec<String> {
    re!(r"\s*(?:,\s*(?:and\s+)?|/|\s+vs\.?\s+|\s+versus\s+|\s+and\s+)\s*").split(s.trim()).map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect()
}

fn property(p: &str, items: &[&'static Feature]) -> Option<Answer> {
    let keys: Vec<&str> = items.iter().filter_map(|f| prop_key(p, f)).collect();
    let key = *keys.first()?;
    let vals: Vec<(String, Option<String>)> = items.iter().map(|f| (f.title(), prop_key(p, f).and_then(|k| value(f, k)))).collect();
    if vals.iter().all(|v| v.1.is_none()) {
        return None;
    }
    let lab = label(key);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} | {lab}", vals.iter().map(|v| v.0.as_str()).collect::<Vec<_>>().join(", "))))];
    if items.len() == 1 {
        pods.push(primary(pod("Result", lab, text(vals[0].1.clone()?))));
        pods.push(pod("Properties", "Basic properties", table(None, rows(items[0]))));
    } else {
        pods.push(primary(pod("Result", lab, text(vals.iter().map(|(a, b)| format!("{a}: {}", b.clone().unwrap_or_else(|| "(not available)".into()))).collect::<Vec<_>>().join(" | ")))));
        let nums: Vec<Option<f64>> = items.iter().map(|f| f.num(key)).collect();
        if items.len() == 2 {
            if let (Some(a), Some(b)) = (nums[0], nums[1]) {
                if b != 0.0 {
                    pods.push(pod("Ratio", "Ratio", text(format!("{} / {} ≈ {}", vals[0].0, vals[1].0, fmt_float(a / b, 4)))));
                }
            }
        }
    }
    Some(Answer::new(lab, pods))
}

const LISTS: &[(&str, &str, &str, bool)] = &[
    (r"highest|tallest", "mountain|volcano", "elev", true),
    (r"highest volcanoes|tallest volcanoes", "volcano", "elev", true),
    (r"longest", "river", "len", true),
    (r"largest|biggest", "lake", "area", true),
    (r"deepest", "lake", "depth", true),
    (r"tallest|highest", "waterfall", "height", true),
    (r"widest", "waterfall", "width", true),
    (r"largest|biggest", "desert", "area", true),
    (r"largest|biggest", "island", "area", true),
    (r"deepest", "trench", "depth", true),
    (r"largest|biggest", "ocean", "area", true),
    (r"deepest", "ocean", "depth", true),
    (r"largest|biggest", "sea", "area", true),
    (r"longest", "cave", "len", true),
    (r"deepest", "cave", "depth", true),
];

fn lists(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+|list of\s+)?(?:(\d+)\s+)?(highest|tallest|longest|largest|biggest|deepest|widest)\s+(mountains|peaks|volcanoes|rivers|lakes|waterfalls|falls|deserts|islands|trenches|ocean trenches|oceans|seas|caves)(?:\s+(?:in|on|of)\s+(?:the\s+)?(?:world|earth))?$").captures(l)?;
    let k: usize = m.get(1).and_then(|x| x.as_str().parse().ok()).unwrap_or(10);
    let kind = match &m[3] {
        "mountains" | "peaks" => "mountain|volcano",
        "volcanoes" => "volcano",
        "rivers" => "river",
        "lakes" => "lake",
        "waterfalls" | "falls" => "waterfall",
        "deserts" => "desert",
        "islands" => "island",
        "trenches" | "ocean trenches" => "trench",
        "oceans" => "ocean",
        "seas" => "sea",
        _ => "cave",
    };
    let (_, _, key, _) = LISTS.iter().find(|(adj, kd, _, _)| Regex::new(&format!("^(?:{adj})$")).ok().is_some_and(|r| r.is_match(&m[2])) && (*kd == kind || kd.split('|').any(|x| x == kind)))?;
    let kinds: Vec<&str> = kind.split('|').collect();
    let mut list: Vec<&Feature> = data().iter().filter(|f| kinds.contains(&f.kind) && f.num(key).is_some()).collect();
    list.sort_by(|a, b| b.num(key).partial_cmp(&a.num(key)).unwrap_or(std::cmp::Ordering::Equal));
    list.truncate(k);
    let lab = label(key);
    let t: Vec<Vec<Content>> = list.iter().enumerate().map(|(i, f)| vec![text(format!("{}", i + 1)), text(f.title()), text(value(f, key).unwrap_or_default()), text(f.place)]).collect();
    let title = format!("{} {}", &m[2], &m[3]);
    Some(Answer::new(title.clone(), vec![
        pod("Input", "Input interpretation", text(title)),
        primary(pod("Result", "Result", text(list.iter().map(|f| f.title()).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Ranking", table(Some(vec!["", "name", lab, "location"]), t)),
        pod("Note", "Note", text("among the features in Noesis's physical geography table")),
    ]))
}

/// Major eruptions: volcano, year, VEI, deaths, note.
const ERUPTIONS: &[(&str, &str, &str, &str, &str)] = &[
    ("Mount Tambora, Indonesia", "1815", "7", "about 71,000", "the deadliest and largest eruption in recorded history; 1816 'year without a summer'"),
    ("Krakatoa, Indonesia", "1883", "6", "about 36,000", "mostly by tsunamis"),
    ("Mount Pelée, Martinique", "1902", "4", "about 28,000", "pyroclastic flows destroyed Saint-Pierre"),
    ("Nevado del Ruiz, Colombia", "1985", "3", "about 23,000", "lahars buried Armero"),
    ("Mount Unzen, Japan", "1792", "—", "about 15,000", "collapse and tsunami"),
    ("Laki, Iceland", "1783–1784", "4", "about 9,000 in Iceland; many more from famine abroad", "fissure eruption; haze over Europe"),
    ("Mount Vesuvius, Italy", "79 AD", "5", "about 2,000 found (Pompeii, Herculaneum)", "buried Pompeii and Herculaneum"),
    ("Mount Pinatubo, Philippines", "1991", "6", "about 850", "second-largest eruption of the 20th century"),
    ("Novarupta, Alaska", "1912", "6", "none", "largest eruption of the 20th century"),
    ("Mount St. Helens, United States", "1980", "5", "57", "lateral blast"),
    ("Hunga Tonga–Hunga Haʻapai, Tonga", "2022", "5–6", "6", "the highest eruption plume on record (57 km)"),
    ("Eyjafjallajökull, Iceland", "2010", "4", "none", "European air traffic stopped for six days"),
    ("Hatepe (Taupō), New Zealand", "about 232 AD", "7", "unknown", ""),
    ("Toba, Sumatra", "about 74,000 years ago", "8", "unknown", "the largest known eruption of the last 2.5 million years"),
];

fn eruptions(l: &str) -> Option<Answer> {
    let one = re!(r"^(?:the\s+)?(?:most destructive|deadliest|worst|largest|biggest) (?:volcanic )?eruptions?(?: in (?:recorded )?history| ever)?$").is_match(l);
    if !one && !re!(r"^(?:major |famous |notable |historic |list of )?(?:volcanic )?eruptions$|^(?:major |famous )?volcanic eruptions in history$").is_match(l) {
        return None;
    }
    let t: Vec<Vec<Content>> = ERUPTIONS.iter().map(|(v, y, vei, d, note)| vec![text(*v), text(*y), text(*vei), text(*d), text(*note)]).collect();
    let main = if one {
        "Mount Tambora, Indonesia, April 1815: VEI 7, about 71,000 deaths (the deadliest eruption in recorded history)".to_string()
    } else {
        ERUPTIONS.iter().take(6).map(|e| format!("{} ({})", e.0, e.1)).collect::<Vec<_>>().join(" | ")
    };
    Some(Answer::new("volcanic eruptions", vec![
        pod("Input", "Input interpretation", text(if one { "most destructive volcanic eruption" } else { "major volcanic eruptions" })),
        primary(pod("Result", "Result", text(main))),
        pod("Table", "Major eruptions (by deaths)", table(Some(vec!["volcano", "year", "VEI", "deaths", "note"]), t)),
        pod("Note", "Note", text("VEI: volcanic explosivity index, a logarithmic scale of erupted volume (0–8)")),
    ]))
}

fn haversine(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (la1, lo1, la2, lo2) = (a.0.to_radians(), a.1.to_radians(), b.0.to_radians(), b.1.to_radians());
    let h = ((la2 - la1) / 2.0).sin().powi(2) + la1.cos() * la2.cos() * ((lo2 - lo1) / 2.0).sin().powi(2);
    2.0 * 6371.0088 * h.sqrt().asin()
}

/// "distance from Blue Grotto to Tunis", "how far is Mount Everest from Kathmandu"
fn distance(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:the\s+)?(?:distance|how far is it|how far)\s+(?:from|between)\s+(?:the\s+)?(.+?)\s+(?:to|and)\s+(?:the\s+)?(.+)$").captures(l)
        .or_else(|| re!(r"^how far is (?:the\s+)?(.+?) from (?:the\s+)?(.+)$").captures(l))?;
    let place = |s: &str| -> Option<(String, (f64, f64), bool)> {
        if let Some(f) = find(s) {
            return Some((f.title(), (f.lat, f.lon), true));
        }
        let c = super::geo::find_city(s)?;
        Some((c.name.clone(), (c.lat, c.lon), false))
    };
    let (a, b) = (place(&m[1])?, place(&m[2])?);
    // leave city-to-city distances to the geography module
    if !a.2 && !b.2 {
        return None;
    }
    let d = haversine(a.1, b.1);
    Some(Answer::new("distance", vec![
        pod("Input", "Input interpretation", text(format!("distance from {} to {}", a.0, b.0))),
        primary(pod("Result", "Great-circle distance", text(format!("{} km ({} miles)", c(d), c(d * 0.621_371))))),
        pod("Note", "Note", text("between the feature's reference point and the city centre")),
    ]))
}

/// "caves in Italy", "volcanoes in Indonesia"
fn in_place(l: &str) -> Option<Answer> {
    let m = re!(r"^(caves|volcanoes|mountains|lakes|rivers|waterfalls|deserts|islands)\s+in\s+(?:the\s+)?(.+)$").captures(l)?;
    let kind = &m[1][..m[1].len() - if m[1].ends_with("oes") { 2 } else { 1 }];
    let kind = if kind == "volcanoe" { "volcano" } else { kind };
    let place = m[2].to_lowercase();
    let list: Vec<&Feature> = data().iter().filter(|f| (f.kind == kind || (kind == "mountain" && f.kind == "volcano")) && f.place.to_lowercase().contains(&place)).collect();
    if list.is_empty() {
        return None;
    }
    let t: Vec<Vec<Content>> = list.iter().map(|f| vec![text(f.title()), text(f.place), text(f.note)]).collect();
    Some(Answer::new(m[0].to_string(), vec![
        pod("Input", "Input interpretation", text(m[0].to_string())),
        primary(pod("Result", "Result", text(list.iter().map(|f| f.title()).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Features", table(Some(vec!["name", "location", "note"]), t)),
        pod("Note", "Note", text("among the features in Noesis's physical geography table")),
    ]))
}

/// "surface area Lake Erie + Lake Huron + Lake Ontario"
fn sum(l: &str) -> Option<Answer> {
    let m = re!(r"^(?:total\s+)?(surface area|area|length|volume|height|elevation)\s+(?:of\s+)?(.+\+.+)$").captures(l)?;
    let items: Option<Vec<&Feature>> = m[2].split('+').map(|s| find_loose(s.trim())).collect();
    let items = items?;
    let key = prop_key(&m[1], items[0])?;
    let total: f64 = items.iter().map(|f| f.num(key)).sum::<Option<f64>>()?;
    let unit = match key { "area" => "km²", "len" => "km", "vol" => "km³", _ => "m" };
    let imp = match key { "area" => format!("{} mi²", c(total * 0.386_102)), "len" => format!("{} mi", c(total * 0.621_371)), "vol" => format!("{} mi³", c(total * 0.239_913)), _ => format!("{} ft", c(total * 3.280_84)) };
    Some(Answer::new("total", vec![
        pod("Input", "Input interpretation", text(format!("{} | {}", items.iter().map(|f| f.title()).collect::<Vec<_>>().join(" + "), label(key)))),
        primary(pod("Result", "Total", text(format!("{} {unit} ({imp})", n(total))))),
        pod("Parts", "Parts", table(None, items.iter().map(|f| vec![text(f.title()), text(value(f, key).unwrap_or_default())]).collect())),
    ]))
}

/// Bare names that mean something else on their own.
const NOT_BARE: &[&str] = &["pacific", "atlantic", "mediterranean", "galapagos", "baikal", "tahoe", "ganga", "lena", "ob", "indus", "amur", "elbrus", "ararat", "kibo", "teide", "etna", "everest", "tambora", "atacama", "mojave", "namib", "gobi", "kalahari", "karakum", "taklamakan", "lascaux", "titicaca", "ladoga", "timpanogos", "east sea", "nantucket", "bali", "crete", "sicily", "corsica", "sardinia", "long island", "great britain", "south island", "honshu", "borneo", "sumatra", "new guinea", "celebes", "sulawesi", "victoria island"];

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.']).replace('’', "'");
    let l = re!(r"\s+").replace_all(&l, " ").to_string();
    let l = re!(r"^(?:what is |what are |what's |where is |tell me about |show |compare )?(?:the )?").replace(&l, "").to_string();
    let l = l.trim_end_matches(" location").trim_end_matches(" wiki summaries").trim_end_matches(" wiki summary").to_string();
    if let Some(a) = lists(&l).or_else(|| eruptions(&l)).or_else(|| distance(&l)).or_else(|| in_place(&l)).or_else(|| sum(&l)) {
        return Some(a);
    }
    // a property: "length Amazon / Nile", "Pacific ocean lowest point", "volcanic arc of mt rainier"
    static FRONT: OnceLock<Regex> = OnceLock::new();
    static BACK: OnceLock<Regex> = OnceLock::new();
    let front = FRONT.get_or_init(|| Regex::new(&format!(r"^({PROPS})(?: of| for| in)? (.+)$")).unwrap());
    let back = BACK.get_or_init(|| Regex::new(&format!(r"^(.+?)(?:'s)? ({PROPS})$")).unwrap());
    let how = re!(r"^(how tall is|how high is|how long is|how wide is|how deep is) (?:the )?(.+)$");
    let pair = how.captures(&l).map(|m| (m[1].to_string(), m[2].to_string()))
        .or_else(|| front.captures(&l).map(|m| (m[1].to_string(), m[2].to_string())))
        .or_else(|| back.captures(&l).map(|m| (m[2].to_string(), m[1].to_string())));
    if let Some((p, items)) = pair {
        let found: Option<Vec<&Feature>> = split_items(&items).iter().map(|s| find_loose(s)).collect();
        if let Some(found) = found.filter(|f| !f.is_empty() && f.len() <= 8) {
            if let Some(a) = property(&p, &found) {
                return Some(a);
            }
        }
    }
    // bare features: "Mt Hood", "Krakatau, Tambora", "Niagara Falls, Yosemite Falls"
    let items = split_items(&l);
    if items.is_empty() || items.len() > 6 {
        return None;
    }
    if items.len() == 1 && NOT_BARE.contains(&norm(&items[0]).as_str()) {
        return None;
    }
    let found: Option<Vec<&Feature>> = if items.len() == 1 { items.iter().map(|s| find(s)).collect() } else { items.iter().map(|s| find_loose(s)).collect() };
    let found = found?;
    if found.len() == 1 {
        Some(card(found[0]))
    } else {
        Some(compare(&found))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        assert_eq!(data().len(), DATA.lines().filter(|l| !l.trim().is_empty()).count());
        for f in data() {
            for (k, v) in &f.props {
                if !matches!(*k, "range" | "arc" | "erupt" | "ascent" | "source" | "mouth" | "river" | "deepest" | "type") {
                    assert!(v.parse::<f64>().is_ok(), "{} {k}={v}", f.names[0]);
                }
            }
        }
        assert!((haversine((0.0, 0.0), (0.0, 1.0)) - 111.2).abs() < 0.1);
    }
}
