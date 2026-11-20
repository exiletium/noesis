//! History: empires, dynasties, historical countries and periods, and
//! battles and wars — dates, duration, region, capital, demonym, the
//! present-day countries they covered, key people, sides, commanders,
//! outcome and losses — plus time spans between historical events.
//! Event dates the curated tables lack come from the Wikidata extract.

use crate::engine::*;
use chrono::NaiveDate;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names | kind | start | end | region | capital | demonym | present-day countries |
/// key people | note | moments ("year: text; year: text").
/// Years are historical: −48 is 48 BCE; "c." marks an approximate date.
const ERAS: &str = "\
holy roman empire,hre|empire|800-12-25|1806-08-06|Central and Western Europe|no fixed capital (Aachen, Prague, Vienna and others)|Holy Roman (German)|Germany, Austria, Czech Republic, Switzerland, Liechtenstein, Belgium, Netherlands, Luxembourg, Slovenia, and parts of France, Italy and Poland|Charlemagne, Otto I, Frederick Barbarossa, Charles V, Francis II|began with Charlemagne's coronation in 800 (or Otto I's in 962); dissolved by Francis II in 1806 during the Napoleonic Wars|962: Otto I crowned emperor; 1356: Golden Bull fixes the election of emperors; 1648: Peace of Westphalia
roman republic|state|-509|-27|Italy and the Mediterranean|Rome|Roman|Italy, Spain, Portugal, France, Greece, Turkey, Tunisia, Libya, Egypt, Syria and others|Julius Caesar, Pompey, Cicero, Cato, Scipio Africanus, Sulla, Marius|ended when the Senate granted Octavian the title Augustus|-264: First Punic War begins; -146: Carthage and Corinth destroyed; -49: Caesar crosses the Rubicon
roman empire,ancient rome|empire|-27|476-09-04|Mediterranean, Western Europe and the Middle East|Rome (Constantinople from 330)|Roman|Italy, Spain, Portugal, France, United Kingdom, Greece, Turkey, Egypt, Tunisia, Syria, Israel and about 40 other countries|Augustus, Nero, Trajan, Hadrian, Marcus Aurelius, Constantine the Great|the Western Empire fell in 476; the Eastern (Byzantine) Empire lasted until 1453|117: greatest extent under Trajan; 330: Constantinople becomes the capital; 395: divided into East and West
byzantine empire,eastern roman empire,byzantium|empire|330-05-11|1453-05-29|Eastern Mediterranean, Balkans and Anatolia|Constantinople|Byzantine (Roman)|Turkey, Greece, Bulgaria, North Macedonia, Albania, Cyprus, Serbia, and at its height also Egypt, Syria, Lebanon, Israel, Jordan, Tunisia, Libya, Italy and southern Spain|Constantine the Great, Justinian I, Theodora, Heraclius, Basil II, Constantine XI|ended when the Ottomans captured Constantinople|555: greatest extent under Justinian I; 1054: East–West Schism; 1204: Fourth Crusade sacks Constantinople
mongol empire,mongolian empire|empire|1206|1368|Eurasia|Karakorum (later Khanbaliq, now Beijing)|Mongol|Mongolia, China, Russia, Kazakhstan, Kyrgyzstan, Uzbekistan, Turkmenistan, Tajikistan, Afghanistan, Iran, Iraq, Syria (briefly), Georgia, Armenia, Azerbaijan, Ukraine, Belarus, Moldova, Korea, Pakistan and eastern Turkey|Genghis Khan, Ögedei Khan, Möngke Khan, Kublai Khan, Batu Khan, Hulagu Khan|the largest contiguous land empire in history (about 24 million km² in 1279); split into four khanates after 1260|1241: invasion of Poland and Hungary; 1258: sack of Baghdad; 1279: conquest of Song China completed
ottoman empire|empire|1299|1922-11-01|Anatolia, the Balkans, the Middle East and North Africa|Söğüt, Bursa, Edirne, then Constantinople (Istanbul) from 1453|Ottoman|Turkey, Greece, Bulgaria, Romania, Serbia, Bosnia and Herzegovina, Albania, North Macedonia, Hungary, Cyprus, Syria, Iraq, Lebanon, Israel, Jordan, Saudi Arabia (Hejaz), Egypt, Libya, Tunisia, Algeria and others|Osman I, Mehmed II, Suleiman the Magnificent, Selim I, Mustafa Kemal Atatürk (successor state)|dissolved after World War I; succeeded by the Republic of Turkey (1923)|1453: conquest of Constantinople; 1529: first siege of Vienna; 1566: death of Suleiman the Magnificent at Szigetvár, with the empire near its greatest extent; 1683: defeat at Vienna
aztec empire,aztecs,triple alliance,the aztecs|empire|1428|1521-08-13|central Mexico|Tenochtitlan|Aztec (Mexica)|Mexico|Itzcoatl, Moctezuma I, Ahuitzotl, Moctezuma II, Cuauhtémoc|the alliance of Tenochtitlan, Texcoco and Tlacopan; fell to Hernán Cortés and his allies after the siege of Tenochtitlan|1519: Cortés arrives
inca empire,incas,the incas,tawantinsuyu|empire|1438|1533|the Andes of South America|Cusco|Inca|Peru, Ecuador, Bolivia, Chile, Argentina, Colombia|Pachacuti, Topa Inca, Huayna Capac, Atahualpa|the largest empire in pre-Columbian America; conquered by Francisco Pizarro (resistance in Vilcabamba until 1572)|1532: Atahualpa captured at Cajamarca
maya civilization,the mayans,mayans,maya,the maya,mayan civilization|civilization|c. -2000|1697|Mesoamerica (the Yucatán Peninsula and surroundings)|city-states such as Tikal, Calakmul, Palenque, Copán and Chichén Itzá|Maya (Mayan)|Mexico, Guatemala, Belize, Honduras, El Salvador|Pakal the Great, Yax Nuun Ahiin I|known for hieroglyphic writing, astronomy and the Long Count calendar; the Classic period was 250–900 CE; the last city, Nojpetén, fell to Spain in 1697|
ancient egypt|civilization|c. -3100|-30|the Nile valley|Memphis, Thebes, Amarna, then Alexandria|Egyptian|Egypt, Sudan|Narmer, Khufu, Hatshepsut, Akhenaten, Tutankhamun, Ramesses II, Cleopatra VII|from the unification under Narmer to the Roman annexation after Cleopatra's death|-2560: Great Pyramid of Giza completed; -1274: Battle of Kadesh
upper egypt|kingdom|c. -3500|c. -3150|the southern Nile valley (from the Delta to the First Cataract)|Nekhen (Hierakonpolis)|Egyptian|Egypt|Scorpion I, Narmer|a predynastic kingdom that ended when Narmer unified Upper and Lower Egypt; afterwards a region of the Egyptian state, symbolized by the white crown|
lower egypt|kingdom|c. -3500|c. -3150|the Nile Delta|Buto|Egyptian|Egypt||a predynastic kingdom united with Upper Egypt under Narmer; symbolized by the red crown|
ancient athens,athens (ancient),classical athens|city-state|c. -508|-146|Attica, Greece|Athens|Athenian|Greece|Cleisthenes, Pericles, Socrates, Plato, Aristotle, Themistocles|dates from the democratic reforms of Cleisthenes to the Roman conquest of Greece|-490: Battle of Marathon; -431: Peloponnesian War begins; -404: defeat by Sparta
sparta,ancient sparta,lacedaemon|city-state|c. -900|-192|Laconia, Peloponnese, Greece|Sparta|Spartan (Lacedaemonian)|Greece|Leonidas I, Lycurgus, Lysander, Agesilaus II|a militaristic city-state with a dual kingship; joined to the Achaean League in 192 BCE|-480: Battle of Thermopylae; -404: victory over Athens in the Peloponnesian War; -371: defeat at Leuctra
polis of chersonesos,chersonesos,chersonesus,chersonese|city-state|c. -422|c. 1400|southwestern Crimea, near Sevastopol|Chersonesos|Chersonesite|Ukraine (Crimea)||an ancient Greek colony of Heraclea Pontica, later a Roman and Byzantine city (Cherson); abandoned in the 15th century; a UNESCO World Heritage Site|
kamakura shogunate,kamakura bakufu,kamakura period|shogunate|1185|1333-07-04|Japan|Kamakura|Japanese|Japan|Minamoto no Yoritomo, Hōjō Masako, Hōjō Tokimune|the first shogunate; Yoritomo was named shōgun in 1192; repelled the Mongol invasions of 1274 and 1281|
tokugawa shogunate,edo period,edo shogunate,japanese edo period|shogunate|1603-03-24|1868-01-03|Japan|Edo (Tokyo)|Japanese|Japan|Tokugawa Ieyasu, Tokugawa Iemitsu, Tokugawa Yoshimune, Tokugawa Yoshinobu, Matsuo Bashō, Katsushika Hokusai, Utagawa Hiroshige|about 250 years of peace and isolation (sakoku); ended with the Meiji Restoration|1853: Commodore Perry's Black Ships arrive
meiji era,meiji period|era|1868-10-23|1912-07-30|Japan|Tokyo|Japanese|Japan|Emperor Meiji, Itō Hirobumi, Ōkubo Toshimichi|rapid industrialization and modernization|
heian period|era|794|1185|Japan|Heian-kyō (Kyoto)|Japanese|Japan|Murasaki Shikibu, Sei Shōnagon, Fujiwara no Michinaga|classical Japanese court culture; The Tale of Genji|
bear flag republic,california republic|republic|1846-06-14|1846-07-09|California|Sonoma|Californian|United States (California)|William B. Ide, Ezekiel Merritt|a 25-day revolt against Mexico; superseded when the US Navy raised the American flag at Sonoma|
songhai empire,songhay empire,songhai,the songhai|empire|1464|1591|West Africa (the Niger River)|Gao|Songhai|Mali, Niger, Nigeria, Senegal, Gambia, Guinea, Burkina Faso, Mauritania|Sunni Ali, Askia Muhammad I|one of the largest African empires; fell to Morocco at the Battle of Tondibi|
benin empire,kingdom of benin|kingdom|1180|1897|West Africa|Benin City (Edo)|Edo (Bini)|Nigeria|Oba Ewuare the Great, Oba Ovonramwen|famous for the Benin Bronzes; annexed by Britain after the 1897 punitive expedition|
ghana empire,ghanaian empire,ghanian empire,wagadu empire,wagadu|empire|c. 300|c. 1200|West Africa (the Sahel)|Koumbi Saleh|Soninke|Mauritania, Mali, Senegal|Kaya Maghan Cissé|grew rich on the trans-Saharan gold and salt trade|
mali empire,manden kurufaba|empire|c. 1235|c. 1670|West Africa|Niani|Malian (Mandinka)|Mali, Senegal, Gambia, Guinea, Guinea-Bissau, Niger, Mauritania, Burkina Faso, Ivory Coast|Sundiata Keita, Mansa Musa|Mansa Musa's 1324 pilgrimage to Mecca displayed its gold wealth; Timbuktu was a centre of learning|1324: Mansa Musa's hajj
kingdom of burgundy,burgundian kingdom,kingdom of the burgundians|kingdom|411|534|the Rhône valley and western Alps|Worms, later Lyon and Geneva|Burgundian|France, Switzerland, Italy|Gundahar, Gundobad, Sigismund|conquered by the Franks; later Kingdoms of Burgundy existed from 888 to 1378|
bernicia|kingdom|c. 547|654|northeast England and southeast Scotland|Bamburgh|Bernician|United Kingdom|Ida, Æthelfrith, Oswald|an Anglo-Saxon kingdom; united with Deira to form Northumbria|
northumbria,kingdom of northumbria|kingdom|654|954|northern England and southeast Scotland|Bamburgh and York|Northumbrian|United Kingdom|Oswald, Oswiu, Edwin, Bede, Eric Bloodaxe|an Anglo-Saxon kingdom formed from Bernicia and Deira; absorbed into England|
han dynasty|dynasty|-202|220|China|Chang'an, then Luoyang|Han Chinese|China, northern Vietnam, northern Korea|Emperor Gaozu, Emperor Wu, Wang Mang (interregnum)|a golden age; opened the Silk Road; gave its name to the Han Chinese|
qin dynasty|dynasty|-221|-206|China|Xianyang|Chinese|China|Qin Shi Huang|the first unified Chinese empire; the Terracotta Army and early Great Wall|
tang dynasty|dynasty|618|907|China|Chang'an|Chinese|China, Mongolia, parts of Central Asia, Korea and Vietnam|Emperor Taizong, Wu Zetian, Li Bai, Du Fu|a cosmopolitan golden age of poetry and trade|
song dynasty|dynasty|960|1279|China|Kaifeng, then Lin'an (Hangzhou)|Chinese|China|Emperor Taizu, Su Shi, Zhu Xi|paper money, gunpowder weapons and movable type; fell to the Mongols|
ming dynasty|dynasty|1368|1644|China|Nanjing, then Beijing|Chinese|China|Hongwu Emperor, Yongle Emperor, Zheng He|built the Forbidden City and most of today's Great Wall|
qing dynasty|dynasty|1644|1912-02-12|China|Beijing|Chinese|China, Mongolia, Taiwan|Kangxi Emperor, Qianlong Emperor, Empress Dowager Cixi, Puyi|the last imperial dynasty of China|
chen dynasty|dynasty|557|589|southern China|Jiankang (Nanjing)|Chinese|China|Chen Baxian, Chen Shubao|the last of the Southern Dynasties; conquered by the Sui, reuniting China|
sui dynasty|dynasty|581|618|China|Daxing (Chang'an), Luoyang|Chinese|China|Emperor Wen, Emperor Yang|reunified China and built the Grand Canal|
gupta empire|empire|c. 320|c. 550|northern India|Pataliputra|Indian|India, Bangladesh, Nepal, Pakistan|Chandragupta I, Samudragupta, Chandragupta II, Kalidasa, Aryabhata|the classical \"golden age\" of India: mathematics (zero and decimal place value), astronomy and Sanskrit literature|400: reign of Chandragupta II (c. 375–415), the empire at its height; the Chinese pilgrim Faxian visits
mughal empire|empire|1526|1857|the Indian subcontinent|Agra, Fatehpur Sikri, Delhi, Lahore|Mughal|India, Pakistan, Bangladesh, Afghanistan|Babur, Akbar, Jahangir, Shah Jahan, Aurangzeb|built the Taj Mahal; ended when Britain deposed Bahadur Shah II|
achaemenid empire,persian empire,first persian empire|empire|-550|-330|Western Asia, Egypt and the Indus valley|Pasargadae, Persepolis, Susa, Babylon|Persian|Iran, Iraq, Turkey, Syria, Egypt, Afghanistan, Pakistan, Israel, Jordan, Lebanon and others|Cyrus the Great, Darius I, Xerxes I|conquered by Alexander the Great|
carthage,ancient carthage,carthaginian empire|city-state|c. -814|-146|North Africa and the western Mediterranean|Carthage|Carthaginian (Punic)|Tunisia, Libya, Algeria, Spain, Italy (Sardinia and Sicily)|Hannibal, Hamilcar Barca, Hasdrubal|destroyed by Rome at the end of the Third Punic War|
minoan civilization,minoans|civilization|c. -3100|c. -1100|Crete and the Aegean islands|Knossos|Minoan|Greece|Minos (legendary)|Europe's first advanced Bronze Age civilization: palaces, Linear A writing and frescoes|-1600: the Neopalatial period, the height of Minoan civilization, around the time of the Thera (Santorini) eruption
mycenaean greece,mycenaeans|civilization|c. -1750|c. -1050|mainland Greece|Mycenae|Mycenaean|Greece|Agamemnon (legendary)|Linear B writing; the setting of Homer's epics|
soviet union,ussr|state|1922-12-30|1991-12-26|Eurasia|Moscow|Soviet|Russia, Ukraine, Belarus, Kazakhstan, Uzbekistan, Turkmenistan, Kyrgyzstan, Tajikistan, Georgia, Armenia, Azerbaijan, Moldova, Lithuania, Latvia, Estonia|Vladimir Lenin, Joseph Stalin, Nikita Khrushchev, Leonid Brezhnev, Mikhail Gorbachev|the first socialist state; dissolved into 15 countries|
stone age|period|c. -3300000|c. -3300|worldwide|—|—|—||from the first stone tools to the start of metalworking; Palaeolithic, Mesolithic and Neolithic|
bronze age|period|c. -3300|c. -1200|the Near East, Europe and Asia (dates vary by region)|—|—|—||the use of bronze tools and weapons, the first cities and writing; ended in the Near East with the Late Bronze Age collapse (c. 1200 BCE)|
iron age|period|c. -1200|c. -550|the Near East and Europe (dates vary by region)|—|—|—||iron tools and weapons replace bronze|
middle ages,medieval period,the middle ages|period|476|1453|Europe|—|—|—|Charlemagne, William the Conqueror, Thomas Aquinas, Joan of Arc, Dante Alighieri|from the fall of the Western Roman Empire to the fall of Constantinople (or Columbus's voyage in 1492)|
renaissance,the renaissance|period|c. 1400|c. 1600|Europe, beginning in Florence, Italy|—|—|Italy, France, the Netherlands, Germany, England, Spain|Leonardo da Vinci, Michelangelo, Raphael, Sandro Botticelli, Niccolò Machiavelli, Petrarch, Johannes Gutenberg|a revival of classical art and learning|
reformation,protestant reformation|period|1517-10-31|1648-10-24|Europe|—|—|Germany, Switzerland, the Netherlands, England, Scotland, Scandinavia|Martin Luther, John Calvin, Huldrych Zwingli, Henry VIII|began with Luther's Ninety-five Theses; ended with the Peace of Westphalia|
scientific revolution|period|1543|1687|Europe|—|—|—|Nicolaus Copernicus, Galileo Galilei, Johannes Kepler, Isaac Newton, Francis Bacon, René Descartes|from Copernicus's De revolutionibus to Newton's Principia|
enlightenment,age of enlightenment,the enlightenment,age of reason|period|c. 1685|c. 1815|Europe (especially France, Britain, Scotland and Germany) and the American colonies|—|—|France, United Kingdom, Germany, Netherlands, United States|Voltaire, Jean-Jacques Rousseau, John Locke, Immanuel Kant, Montesquieu, Denis Diderot, David Hume, Adam Smith, Benjamin Franklin|an intellectual movement of reason, science, liberty and religious tolerance|
industrial revolution,the industrial revolution,first industrial revolution|period|c. 1760|c. 1840|Great Britain, then Western Europe and the United States|—|—|United Kingdom, Belgium, France, Germany, United States|James Watt, Richard Arkwright, George Stephenson, Isambard Kingdom Brunel, Eli Whitney, Matthew Boulton|the shift from hand production to machines, steam power and factories|
romantic period,romanticism,romantic era|period|c. 1800|c. 1850|Europe|—|—|Germany, United Kingdom, France|Ludwig van Beethoven, Frédéric Chopin, William Wordsworth, Lord Byron, Percy Bysshe Shelley, John Keats, Johann Wolfgang von Goethe, Victor Hugo, Mary Shelley, Caspar David Friedrich, Eugène Delacroix|emphasis on emotion, nature and the individual in art, music and literature|
tudor period,tudor era|era|1485-08-22|1603-03-24|England and Wales|London|English|United Kingdom|Henry VII, Henry VIII, Edward VI, Mary I, Elizabeth I|the reign of the House of Tudor|
elizabethan era|era|1558-11-17|1603-03-24|England|London|English|United Kingdom|Elizabeth I, William Shakespeare, Christopher Marlowe, Francis Drake, Walter Raleigh|the reign of Elizabeth I; a golden age of English drama|1588: defeat of the Spanish Armada
jacobean era|era|1603-03-24|1625-03-27|England and Scotland|London|English|United Kingdom|James I, William Shakespeare, Ben Jonson, Francis Bacon|the reign of James VI and I; the King James Bible (1611)|1605: Gunpowder Plot
georgian era|era|1714-08-01|1837-06-20|Great Britain|London|British|United Kingdom|George I, George II, George III, George IV, William IV, Jane Austen|the reigns of the four Georges and William IV|
victorian era|era|1837-06-20|1901-01-22|United Kingdom and the British Empire|London|British|United Kingdom|Queen Victoria, Charles Dickens, Charles Darwin, Benjamin Disraeli, William Gladstone|the reign of Queen Victoria|
edwardian era|era|1901-01-22|1910-05-06|United Kingdom|London|British|United Kingdom|Edward VII|the reign of Edward VII|
bell beaker culture,bell beaker,beaker culture|culture|c. -2800|c. -1800|western and central Europe|—|Beaker folk|Spain, Portugal, France, United Kingdom, Ireland, Netherlands, Germany, Czech Republic, Hungary and others||named after its distinctive inverted-bell drinking vessels; spread copper metallurgy across Europe|
gilded age|era|c. 1870|c. 1900|United States|—|American|United States|Andrew Carnegie, John D. Rockefeller, J. P. Morgan, Cornelius Vanderbilt|rapid industrial growth and great wealth inequality; named by Mark Twain|
";

/// names | war | start | end | place | sides | commanders | strength | losses | outcome
const BATTLES: &str = "\
battle of fort sumter,fort sumter|American Civil War|1861-04-12|1861-04-13|Charleston Harbor, South Carolina|Confederate States vs United States (Union)|P. G. T. Beauregard; Robert Anderson|Confederates: about 500; Union: 85 soldiers|no deaths in combat; 1 Union soldier killed and 1 mortally wounded by an accidental explosion during the surrender salute|Confederate victory; the first battle of the American Civil War
battle of appomattox court house,appomattox court house,battle of appomattox|American Civil War|1865-04-09|1865-04-09|Appomattox County, Virginia|United States (Union) vs Confederate States|Ulysses S. Grant; Robert E. Lee|Union: about 63,000; Confederates: about 28,000|Union: 164; Confederates: about 500 killed or wounded|Union victory; Lee surrendered the Army of Northern Virginia
first battle of bull run,first bull run,first battle of manassas|American Civil War|1861-07-21|1861-07-21|Prince William County, Virginia|Confederate States vs United States (Union)|P. G. T. Beauregard, Joseph E. Johnston; Irvin McDowell|Confederates: about 34,000; Union: about 35,000|Union: about 2,900; Confederates: about 1,980|Confederate victory
battle of shiloh,shiloh|American Civil War|1862-04-06|1862-04-07|Hardin County, Tennessee|United States (Union) vs Confederate States|Ulysses S. Grant, Don Carlos Buell; Albert Sidney Johnston, P. G. T. Beauregard|Union: about 66,000; Confederates: about 44,000|about 23,700 total (3,500 killed)|Union victory
battle of antietam,antietam,battle of sharpsburg|American Civil War|1862-09-17|1862-09-17|Sharpsburg, Maryland|United States (Union) vs Confederate States|George B. McClellan; Robert E. Lee|Union: about 87,000; Confederates: about 38,000|about 22,700 total; the bloodiest single day in American history|inconclusive tactically; strategic Union victory that led to the Emancipation Proclamation
battle of fredericksburg|American Civil War|1862-12-11|1862-12-15|Fredericksburg, Virginia|Confederate States vs United States (Union)|Robert E. Lee; Ambrose Burnside|Confederates: about 72,000; Union: about 114,000|Union: about 12,650; Confederates: about 5,300|Confederate victory
battle of chancellorsville|American Civil War|1863-04-30|1863-05-06|Spotsylvania County, Virginia|Confederate States vs United States (Union)|Robert E. Lee, Stonewall Jackson; Joseph Hooker|Confederates: about 61,000; Union: about 133,000|Union: about 17,300; Confederates: about 13,300 (including Stonewall Jackson, mortally wounded)|Confederate victory
battle of gettysburg,gettysburg|American Civil War|1863-07-01|1863-07-03|Gettysburg, Pennsylvania|United States (Union) vs Confederate States|George G. Meade; Robert E. Lee|Union: about 104,000; Confederates: about 75,000|about 51,000 total casualties (Union 23,049; Confederates 23,000–28,000)|Union victory; the turning point of the war in the East
siege of vicksburg,vicksburg|American Civil War|1863-05-18|1863-07-04|Vicksburg, Mississippi|United States (Union) vs Confederate States|Ulysses S. Grant; John C. Pemberton|Union: about 77,000; Confederates: about 33,000|Union: about 4,800; Confederates: about 3,200 plus 29,500 surrendered|Union victory; gave the Union control of the Mississippi River
battle of chickamauga|American Civil War|1863-09-18|1863-09-20|Catoosa and Walker Counties, Georgia|Confederate States vs United States (Union)|Braxton Bragg, James Longstreet; William Rosecrans|Confederates: about 65,000; Union: about 60,000|about 34,600 total|Confederate victory
battle of pharsalus,pharsalus|Caesar's Civil War|-48-08-09|-48-08-09|Pharsalus, Thessaly, Greece|Caesarians vs Optimates (Pompeians)|Julius Caesar, Mark Antony, Publius Sulla; Pompey the Great, Titus Labienus, Metellus Scipio|Caesar: about 22,000 infantry and 1,000 cavalry; Pompey: about 45,000 infantry and 7,000 cavalry|Caesar: about 200–1,200 killed; Pompey: about 6,000–15,000 killed, 24,000 captured|decisive Caesarian victory
battle of kadesh,kadesh|Egyptian–Hittite wars|-1274-05-01|-1274-05-01|Kadesh, on the Orontes River (Syria)|New Kingdom of Egypt vs Hittite Empire|Ramesses II; Muwatalli II|Egypt: about 20,000; Hittites: about 23,000–50,000|unknown|inconclusive; both sides claimed victory; probably the largest chariot battle ever fought (5,000–6,000 chariots)
battle of actium,actium|Final War of the Roman Republic|-31-09-02|-31-09-02|Actium, Ionian Sea, Greece|Octavian vs Mark Antony and Ptolemaic Egypt|Octavian, Marcus Agrippa; Mark Antony, Cleopatra VII|Octavian: about 400 ships; Antony and Cleopatra: about 230–290 ships|Antony: about 5,000 killed and 200 ships sunk or captured|decisive victory for Octavian, who became Augustus
battle of zama,zama|Second Punic War|-202-10-19|-202-10-19|Zama, near Siliana, Tunisia|Roman Republic and Numidia vs Carthage|Scipio Africanus, Masinissa, Gaius Laelius; Hannibal Barca|Rome: about 29,000 infantry and 6,100 cavalry; Carthage: about 36,000 infantry, 4,000 cavalry and 80 war elephants|Rome: about 2,500 killed; Carthage: about 20,000 killed and 20,000 captured|decisive Roman victory that ended the Second Punic War
battle of megiddo|Egyptian campaigns in Canaan|c. -1457|c. -1457|Megiddo, Canaan (northern Israel)|New Kingdom of Egypt vs Canaanite coalition led by Kadesh|Thutmose III; the king of Kadesh|Egypt: 10,000–20,000|unknown|Egyptian victory; the first battle recorded in relative detail
battle of culloden,culloden|Jacobite rising of 1745|1746-04-16|1746-04-16|Culloden Moor, near Inverness, Scotland|Great Britain vs Jacobites|Prince William, Duke of Cumberland; Charles Edward Stuart (Bonnie Prince Charlie), Lord George Murray|Government: about 8,000; Jacobites: about 5,000–6,000|Government: about 300; Jacobites: about 1,500–2,000 killed or wounded|decisive government victory; the last pitched battle on British soil
wars of the roses,war of the roses|Wars of the Roses|1455-05-22|1487-06-16|England|House of Lancaster vs House of York|Henry VI, Margaret of Anjou, Henry Tudor; Richard of York, Edward IV, Richard III|—|about 28,000 killed in battle (estimates vary widely)|Lancastrian-Tudor victory: Henry VII founded the Tudor dynasty
tet offensive|Vietnam War|1968-01-30|1968-09-23|South Vietnam|North Vietnam and the Viet Cong vs South Vietnam, the United States and allies|Võ Nguyên Giáp, Văn Tiến Dũng; William Westmoreland, Cao Văn Viên|communist forces: about 80,000 in the first phase|North Vietnam and Viet Cong: about 45,000 killed (US estimate); US: about 4,000 killed; South Vietnam: about 5,000 killed; about 14,000 civilians killed|military defeat for North Vietnam but a political and strategic turning point that eroded US support for the war
battle of okinawa,okinawa,operation iceberg|World War II (Pacific War)|1945-04-01|1945-06-22|Okinawa, Ryukyu Islands, Japan|United States and the United Kingdom vs Japan|Simon Bolivar Buckner Jr., Chester Nimitz; Mitsuru Ushijima|Allies: about 541,000 (183,000 combat troops); Japan: about 116,000|US: about 12,500 killed; Japan: about 77,000–110,000 soldiers killed; 40,000–150,000 Okinawan civilians killed. Materiel: US lost 36 ships sunk, 368 damaged and 763 aircraft; Japan lost 16 ships, about 7,800 aircraft, 27 tanks and 743 artillery pieces|Allied victory; the bloodiest battle of the Pacific War
operation desert storm,desert storm,gulf war,persian gulf war|Gulf War|1991-01-17|1991-02-28|Iraq, Kuwait and Saudi Arabia|US-led coalition of 35 countries vs Iraq|Norman Schwarzkopf, Colin Powell, Khalid bin Sultan; Saddam Hussein|coalition: about 956,000; Iraq: about 650,000|coalition: 292 killed (147 in combat); Iraq: about 20,000–35,000 killed. Iraqi equipment destroyed: about 3,300 tanks, 2,100 armoured personnel carriers and 2,200 artillery pieces|coalition victory; Kuwait liberated
battle of dunkirk,dunkirk,operation dynamo|World War II|1940-05-26|1940-06-04|Dunkirk, France|United Kingdom, France and Belgium vs Germany|Lord Gort, Bertram Ramsay, Jean-Marie Abrial; Gerd von Rundstedt, Ewald von Kleist|Allies: about 400,000; Germany: about 800,000|338,226 Allied soldiers evacuated; about 34,000 French soldiers captured|German victory, but the evacuation saved most of the British Expeditionary Force
second crusade|Crusades|1147|1149|the Levant, Anatolia, Iberia and the Baltic|Crusaders (France, the Holy Roman Empire, the Crusader states, Portugal) vs Seljuk Turks, Zengids and Almoravids|Louis VII of France, Conrad III of Germany, Baldwin III of Jerusalem; Nur ad-Din, Mu'in ad-Din Unur|—|heavy crusader losses in Anatolia|crusader defeat in the East after the failed siege of Damascus (1148); crusader success at Lisbon (1147)
first crusade|Crusades|1096|1099|Anatolia and the Levant|Crusaders vs Seljuk Turks and the Fatimid Caliphate|Godfrey of Bouillon, Raymond of Toulouse, Bohemond of Taranto; Kilij Arslan I, Iftikhar al-Dawla|—|—|crusader victory; capture of Jerusalem (1099) and founding of the Crusader states
roman–persian wars,roman-persian wars,persian-roman wars,persian–roman wars|Roman–Persian Wars|-54|628|Mesopotamia, Armenia, the Caucasus and Syria|Roman Republic, Roman Empire and Byzantine Empire vs Parthian and Sasanian empires|Crassus, Trajan, Julian, Heraclius; Surena, Shapur I, Khosrow II|—|—|status quo ante bellum; both empires were exhausted shortly before the Arab conquests; one of the longest conflicts in history (about 680 years)
peruvian war of independence|Spanish American wars of independence|1809|1826|Peru|Peruvian, Argentine, Chilean and Gran Colombian patriots vs the Spanish Empire|José de San Martín, Simón Bolívar, Antonio José de Sucre; José de la Serna|—|—|independence of Peru (declared 28 July 1821; secured at Ayacucho, 1824)
cambodian civil war|Cold War / Vietnam War|1967|1975-04-17|Cambodia|Khmer Rouge, North Vietnam and the Viet Cong vs the Kingdom of Cambodia, then the Khmer Republic, supported by the United States and South Vietnam|Pol Pot, Ieng Sary; Lon Nol, Sisowath Sirik Matak|—|about 275,000–310,000 deaths|Khmer Rouge victory; capture of Phnom Penh and founding of Democratic Kampuchea
soviet–japanese border war,soviet-japanese border war,soviet-japanese border conflicts|Soviet–Japanese border conflicts|1932|1939-09-16|Manchuria, Mongolia and the Soviet Far East|Soviet Union and Mongolia vs Japan and Manchukuo|Georgy Zhukov, Grigori Shtern; Michitarō Komatsubara|—|—|Soviet–Mongolian victory at Khalkhin Gol (1939); ceasefire and the 1941 Soviet–Japanese neutrality pact
war of the pacific,saltpeter war|War of the Pacific|1879-04-05|1884-10-20|the Atacama Desert and the Pacific coast of South America|Chile vs Peru and Bolivia|Manuel Baquedano, Patricio Lynch; Miguel Grau, Andrés Avelino Cáceres, Hilarión Daza|—|about 14,000–23,000 killed|Chilean victory; Bolivia lost its coastline and Peru the province of Tarapacá
";

/// Events missing from the Wikidata extract: names | start | end | place | note
const EVENTS: &str = "\
great baltimore fire,baltimore fire|1904-02-07|1904-02-08|Baltimore, Maryland, United States|destroyed more than 1,500 buildings over about 30 hours in central Baltimore
first powered flight,wright brothers first flight,first flight|1903-12-17|1903-12-17|Kill Devil Hills, near Kitty Hawk, North Carolina, United States|the Wright Flyer, piloted by Orville Wright, flew 37 m (120 ft) in 12 seconds; the fourth flight that day, by Wilbur, covered 260 m in 59 seconds
diet of worms|1521-01-28|1521-05-25|Worms, Germany|imperial assembly at which Martin Luther refused to recant; the Edict of Worms declared him an outlaw
lindbergh's solo flight,lindbergh solo flight,lindbergh flight|1927-05-20|1927-05-21|from Roosevelt Field, New York, to Le Bourget, Paris|the first solo non-stop transatlantic flight: 33.5 hours in the Spirit of St. Louis
";

/// Notable events by date, for "what happened in August 1969".
const TIMELINE: &str = "1776-07-04|the Second Continental Congress adopts the Declaration of Independence
1789-07-14|storming of the Bastille, Paris
1912-04-15|RMS Titanic sinks after striking an iceberg
1914-06-28|Archduke Franz Ferdinand assassinated in Sarajevo
1929-10-29|Black Tuesday: the Wall Street stock market crash
1941-12-07|Japanese attack on Pearl Harbor
1944-06-06|D-Day: Allied landings in Normandy
1945-05-08|V-E Day: Germany's surrender takes effect
1945-08-06|atomic bombing of Hiroshima
1945-08-09|atomic bombing of Nagasaki
1957-10-04|Sputnik 1, the first artificial satellite, launched
1961-04-12|Yuri Gagarin becomes the first human in space
1963-11-22|President John F. Kennedy assassinated in Dallas
1969-06-28|Stonewall riots begin in New York City
1969-07-16|Apollo 11 launches from Kennedy Space Center
1969-07-18|Chappaquiddick incident (Ted Kennedy's car accident)
1969-07-20|Apollo 11 lands on the Moon; Neil Armstrong and Buzz Aldrin walk on the surface
1969-08-08|the Beatles photographed crossing Abbey Road for the album cover
1969-08-09|Manson Family murders of Sharon Tate and others in Los Angeles
1969-08-14|British troops deployed in Northern Ireland (Operation Banner)
1969-08-15|Woodstock music festival begins in Bethel, New York (to August 18)
1969-08-17|Hurricane Camille makes landfall on the Mississippi coast as a Category 5 storm
1969-10-29|the first ARPANET message is sent, from UCLA to SRI
1986-01-28|Space Shuttle Challenger breaks apart after launch
1986-04-26|Chernobyl nuclear disaster
1989-03-24|Exxon Valdez oil spill in Prince William Sound, Alaska
1989-06-04|Tiananmen Square crackdown in Beijing
1989-10-17|Loma Prieta earthquake in the San Francisco Bay Area
1989-11-09|fall of the Berlin Wall: East Germany opens its border crossings
1989-11-10|Todor Zhivkov removed as leader of Bulgaria
1989-11-17|Velvet Revolution begins with a student demonstration in Prague
1989-12-20|United States invasion of Panama
1989-12-25|Nicolae and Elena Ceaușescu executed in Romania
1990-10-03|German reunification
1991-12-26|the Soviet Union is dissolved
2001-09-09|Afghan resistance leader Ahmad Shah Massoud assassinated
2001-09-11|September 11 attacks on the World Trade Center and the Pentagon
2001-09-18|first anthrax letters mailed in the United States
2001-10-07|US-led invasion of Afghanistan begins (Operation Enduring Freedom)
2001-10-23|Apple announces the iPod
2001-10-25|Microsoft releases Windows XP
2001-10-26|USA PATRIOT Act signed into law
2004-12-26|Indian Ocean earthquake and tsunami
2008-09-15|Lehman Brothers files for bankruptcy
2009-01-15|US Airways Flight 1549 lands on the Hudson River
2009-01-20|Barack Obama inaugurated as US president
2009-05-18|Sri Lanka's government declares victory over the Tamil Tigers, ending the civil war
2009-06-11|WHO declares the H1N1 swine flu pandemic
2009-06-25|Michael Jackson dies
2011-03-11|Tōhoku earthquake and tsunami; Fukushima nuclear disaster";

/// "What happened in August 1969?", "events from November 1, 1989 to November 30, 1989", "May 18, 2009 events"
fn timeline(l: &str) -> Option<Answer> {
    const MONTHS: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];
    let month = |s: &str| MONTHS.iter().position(|m| s.len() >= 3 && m.starts_with(s.trim_end_matches('.'))).map(|i| i as u32 + 1);
    let body = re!(r"^(?:notable |major |historical |important )?(?:events (?:on|in|from|during|between) |events |what happened (?:on|in|during) |what happened |happenings (?:on|in) )(.+)$").captures(l).map(|m| m[1].to_string())
        .or_else(|| re!(r"^(.+?) (?:events|happenings|in history)$").captures(l).map(|m| m[1].to_string()))?;
    let date = re!(r"^([a-z]+)\.? (\d{1,2})(?:st|nd|rd|th)?(?:,? (\d{4}))?$");
    let month_year = re!(r"^([a-z]+) (\d{4})$");
    let (lo, hi, label): ((i32, u32, u32), (i32, u32, u32), String) = if let Some(m) = re!(r"^(.+?) (?:to|through|until|and|-|–) (.+)$").captures(&body) {
        let b = date.captures(m[2].trim())?;
        let a = date.captures(m[1].trim())?;
        let yb: i32 = b.get(3)?.as_str().parse().ok()?;
        let ya: i32 = a.get(3).map_or(Some(yb), |y| y.as_str().parse().ok())?;
        let (ma, mb) = (month(&a[1])?, month(&b[1])?);
        ((ya, ma, a[2].parse().ok()?), (yb, mb, b[2].parse().ok()?), format!("{} {}, {ya} to {} {}, {yb}", cap(MONTHS[ma as usize - 1]), &a[2], cap(MONTHS[mb as usize - 1]), &b[2]))
    } else if let Some(m) = date.captures(&body) {
        let y: i32 = m.get(3)?.as_str().parse().ok()?;
        let mo = month(&m[1])?;
        let d: u32 = m[2].parse().ok()?;
        ((y, mo, d), (y, mo, d), format!("{} {d}, {y}", cap(MONTHS[mo as usize - 1])))
    } else if let Some(m) = month_year.captures(&body) {
        let y: i32 = m[2].parse().ok()?;
        let mo = month(&m[1])?;
        ((y, mo, 1), (y, mo, 31), format!("{} {y}", cap(MONTHS[mo as usize - 1])))
    } else if let Some(m) = re!(r"^(?:the year )?(\d{4})$").captures(&body) {
        let y: i32 = m[1].parse().ok()?;
        ((y, 1, 1), (y, 12, 31), y.to_string())
    } else {
        return None;
    };
    let rows: Vec<(String, &str)> = TIMELINE
        .lines()
        .filter_map(|r| {
            let (d, e) = r.split_once('|')?;
            let p: Vec<i64> = d.split('-').filter_map(|x| x.parse().ok()).collect();
            let k = (p[0] as i32, p[1] as u32, p[2] as u32);
            (k >= lo && k <= hi).then(|| (d.to_string(), e))
        })
        .collect();
    if rows.is_empty() {
        return None;
    }
    let shown = |d: &str| parse(d, false).map(show).unwrap_or_else(|| d.to_string());
    Some(Answer::new(format!("events {label}"), vec![
        input(format!("notable events | {label}")),
        primary(pod("Result", "Notable events", text(rows.iter().map(|(d, e)| format!("{}: {e}", shown(d))).collect::<Vec<_>>().join(" | ")))),
        pod("Table", "Timeline", table(Some(vec!["date", "event"]), rows.iter().map(|(d, e)| vec![text(shown(d)), text(*e)]).collect())),
        pod("Note", "Note", text("a selection of notable events, not a complete chronology")),
    ]))
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
}

/// A historical date in astronomical years (1 BCE = 0).
#[derive(Clone, Copy)]
struct D {
    y: i32,
    m: u32,
    d: u32,
    approx: bool,
}

/// Parses "1846-06-14", "-48-08-09" (historical BCE) or "c. 1400"; `astro`
/// marks Wikidata dates, which already use astronomical years.
fn parse(s: &str, astro: bool) -> Option<D> {
    let s = s.trim();
    let (approx, s) = match s.strip_prefix("c. ") { Some(r) => (true, r), None => (false, s) };
    let neg = s.starts_with('-');
    let mut it = s.trim_start_matches('-').split('-');
    let y: i32 = it.next()?.parse().ok()?;
    let y = if neg { if astro { -y } else { 1 - y } } else { y };
    let m = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    let d = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
    Some(D { y, m, d, approx })
}

fn year_text(y: i32) -> String {
    if y <= 0 { format!("{} BCE", 1 - y) } else if y < 1000 { format!("{y} CE") } else { y.to_string() }
}

fn show(d: D) -> String {
    let c = if d.approx { "c. " } else { "" };
    if d.m > 0 && d.d > 0 {
        let month = ["", "January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"][d.m as usize];
        if d.y >= 1583 {
            if let Some(dt) = NaiveDate::from_ymd_opt(d.y, d.m, d.d) {
                return format!("{c}{}", dt.format("%A, %B %-d, %Y"));
            }
        }
        return format!("{c}{month} {}, {}", d.d, year_text(d.y));
    }
    format!("{c}{}", year_text(d.y))
}

/// The time from `a` to `b`.
fn span(a: D, b: D) -> String {
    if a.m > 0 && a.d > 0 && b.m > 0 && b.d > 0 && a.y > 0 && b.y > 0 && !a.approx && !b.approx {
        if let (Some(x), Some(y)) = (NaiveDate::from_ymd_opt(a.y, a.m, a.d), NaiveDate::from_ymd_opt(b.y, b.m, b.d)) {
            let days = (y - x).num_days();
            let mut years = b.y - a.y;
            if (b.m, b.d) < (a.m, a.d) {
                years -= 1;
            }
            return if years >= 1 { format!("{years} years ({} days)", group(days)) } else { format!("{days} days") };
        }
    }
    let y = b.y - a.y;
    format!("{}{} years", if a.approx || b.approx || a.m == 0 || b.m == 0 { "about " } else { "" }, group(y as i64))
}

fn group(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("−{out}") } else { out }
}

fn eras() -> &'static Vec<Vec<&'static str>> {
    static E: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    E.get_or_init(|| ERAS.lines().map(|l| l.split('|').collect()).collect())
}

fn extras() -> &'static Vec<Vec<&'static str>> {
    static X: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    X.get_or_init(|| EVENTS.lines().map(|l| l.split('|').collect()).collect())
}

fn battles() -> &'static Vec<Vec<&'static str>> {
    static B: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    B.get_or_init(|| BATTLES.lines().map(|l| l.split('|').collect()).collect())
}

fn title(s: &str) -> String {
    let small = ["of", "the", "and", "de", "von"];
    s.split(' ').enumerate().map(|(i, w)| {
        if i > 0 && small.contains(&w) { return w.to_string(); }
        w.split('-').map(|p| p.split('–').map(|q| { let mut c = q.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).collect::<Vec<_>>().join("–")).collect::<Vec<_>>().join("-")
    }).collect::<Vec<_>>().join(" ").replace("Hre", "HRE").replace("Ussr", "USSR")
}

/// Something with a start and end: a curated era or battle, or a Wikidata event.
enum Thing {
    Era(&'static Vec<&'static str>),
    Battle(&'static Vec<&'static str>),
    Event(&'static str, Option<D>, Option<D>),
    /// curated supplementary events: the row
    Extra(&'static Vec<&'static str>),
}

impl Thing {
    fn name(&self) -> String {
        match self {
            Thing::Era(r) | Thing::Battle(r) => title(r[0].split(',').next().unwrap_or("")),
            Thing::Event(n, ..) => n.to_string(),
            Thing::Extra(r) => title(r[0].split(',').next().unwrap_or("")),
        }
    }
    fn start(&self) -> Option<D> {
        match self {
            Thing::Era(r) => parse(r[2], false),
            Thing::Battle(r) => parse(r[2], false),
            Thing::Event(_, s, _) => *s,
            Thing::Extra(r) => parse(r[1], false),
        }
    }
    fn end(&self) -> Option<D> {
        match self {
            Thing::Era(r) => parse(r[3], false),
            Thing::Battle(r) => parse(r[3], false),
            Thing::Event(_, _, e) => *e,
            Thing::Extra(r) => parse(r[2], false),
        }
    }
}

fn find(name: &str, allow_events: bool) -> Option<Thing> {
    let n = name.trim().trim_start_matches("the ").trim();
    let hit = |r: &&Vec<&str>| r[0].split(',').any(|x| x == n || x.trim_start_matches("the ") == n);
    if let Some(r) = eras().iter().find(hit) {
        return Some(Thing::Era(r));
    }
    if let Some(r) = battles().iter().find(hit) {
        return Some(Thing::Battle(r));
    }
    if let Some(r) = extras().iter().find(hit) {
        return Some(Thing::Extra(r));
    }
    // "founding of Carthage", "fall of the Aztec empire"
    if let Some(m) = re!(r"^(founding|foundation|beginning|start|rise|fall|collapse|end|dissolution) of (?:the )?(.+)$").captures(n) {
        if let Some(t) = find(&m[2], false) {
            if matches!(t, Thing::Era(_)) {
                let begin = matches!(&m[1], "founding" | "foundation" | "beginning" | "start" | "rise");
                let d = if begin { t.start() } else { t.end() };
                let name: &'static str = Box::leak(format!("{} of {}", &m[1], t.name()).into_boxed_str());
                return Some(Thing::Event(name, d, d));
            }
        }
    }
    // Wikidata events: on their own only for names of two or more words
    if !allow_events && !n.contains(' ') {
        return None;
    }
    let (en, s, e, p) = super::facts::event(n)?;
    if !allow_events && s.is_empty() && e.is_empty() && p.is_empty() {
        return None;
    }
    // a treaty's point in time is its signing; its start is entry into force
    let (s, e) = match (s.is_empty(), e.is_empty(), p.is_empty()) {
        (false, true, false) => (p, p),
        (true, _, _) => (p, if e.is_empty() { p } else { e }),
        (false, true, true) => (s, s),
        _ => (s, e),
    };
    Some(Thing::Event(en, parse(s, true), parse(e, true)))
}

fn input(s: impl Into<String>) -> Pod {
    pod("Input", "Input interpretation", text(s.into()))
}

fn era_rows(r: &[&str]) -> Vec<Vec<Content>> {
    let (s, e) = (parse(r[2], false), parse(r[3], false));
    let mut v = vec![vec![text("type"), text(r[1])]];
    if let (Some(s), Some(e)) = (s, e) {
        v.push(vec![text("start"), text(show(s))]);
        v.push(vec![text("end"), text(show(e))]);
        v.push(vec![text("duration"), text(span(s, e))]);
    }
    for (i, lab) in [(4, "region"), (5, "capital"), (6, "demonym"), (7, "present-day countries"), (8, "key people"), (9, "note")] {
        if !r[i].is_empty() && r[i] != "—" {
            v.push(vec![text(lab), text(r[i])]);
        }
    }
    v
}

fn battle_rows(r: &[&str]) -> Vec<Vec<Content>> {
    let (s, e) = (parse(r[2], false), parse(r[3], false));
    let mut v = vec![vec![text("part of"), text(r[1])]];
    if let (Some(s), Some(e)) = (s, e) {
        if r[2] == r[3] {
            v.push(vec![text("date"), text(show(s))]);
        } else {
            v.push(vec![text("start"), text(show(s))]);
            v.push(vec![text("end"), text(show(e))]);
            v.push(vec![text("duration"), text(span(s, e))]);
        }
    }
    for (i, lab) in [(4, "location"), (5, "belligerents"), (6, "commanders"), (7, "strength"), (8, "casualties and losses"), (9, "result")] {
        if !r[i].is_empty() && r[i] != "—" {
            v.push(vec![text(lab), text(r[i])]);
        }
    }
    v
}

fn card(t: &Thing) -> Option<Answer> {
    match t {
        Thing::Era(r) => Some(Answer::new("history", vec![
            input(format!("{} | {}", t.name(), r[1])),
            primary(pod("Result", "Summary", text(format!("{} in {} ({} to {})", capital(r[1]), r[4], show(t.start()?), show(t.end()?))))),
            pod("Properties", "Basic information", table(None, era_rows(r))),
        ])),
        Thing::Battle(r) => Some(Answer::new("history", vec![
            input(format!("{} | military conflict", t.name())),
            primary(pod("Result", "Result", text(r[9]))),
            pod("Properties", "Basic information", table(None, battle_rows(r))),
        ])),
        Thing::Extra(r) => {
            let (s, e) = (t.start()?, t.end()?);
            let same = r[1] == r[2];
            let when = if same { show(s) } else { format!("{} to {}", show(s), show(e)) };
            Some(Answer::new("history", vec![
                input(format!("{} | event", t.name())),
                primary(pod("Result", "Date", text(when.clone()))),
                pod("Properties", "Basic information", table(None, vec![
                    vec![text(if same { "date" } else { "dates" }), text(when)],
                    vec![text("location"), text(r[3])],
                    vec![text("description"), text(r[4])],
                ])),
            ]))
        }
        Thing::Event(..) => {
            let (s, e) = (t.start()?, t.end()?);
            let same = s.y == e.y && s.m == e.m && s.d == e.d;
            let mut rows = vec![];
            if same {
                rows.push(vec![text("date"), text(show(s))]);
            } else {
                rows.push(vec![text("start"), text(show(s))]);
                rows.push(vec![text("end"), text(show(e))]);
                rows.push(vec![text("duration"), text(span(s, e))]);
            }
            Some(Answer::new("history", vec![
                input(format!("{} | event", t.name())),
                primary(pod("Result", "Dates", text(if same { show(s) } else { format!("{} to {}", show(s), show(e)) }))),
                pod("Properties", "Dates", table(None, rows)),
                pod("Source", "Source", text("Wikidata (CC0)")),
            ]))
        }
    }
}

fn split(s: &str) -> Vec<&str> {
    re!(r"\s*(?:,\s*(?:and\s+)?|\s+vs\.?\s+|\s+versus\s+|\s+and\s+(?:the\s+)?)\s*").split(s).map(|x| x.trim()).filter(|x| !x.is_empty()).collect()
}

/// "the start of X", "X start date", "the end of X", "X" → (thing, which end)
fn endpoint(s: &str) -> Option<(Thing, Option<D>, String)> {
    let s = s.trim().trim_start_matches("the ");
    let (which, name) = if let Some(m) = re!(r"^(start|beginning|end|finish)(?: date)? of (?:the )?(.+)$").captures(s) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = re!(r"^(.+?) (start|end|beginning)(?: date)?$").captures(s) {
        (m[2].to_string(), m[1].to_string())
    } else {
        ("start".to_string(), s.to_string())
    };
    let t = find(&name, true)?;
    let d = if which == "end" || which == "finish" { t.end() } else { t.start() };
    let label = format!("{} of the {}", if which == "end" || which == "finish" { "end" } else { "start" }, t.name());
    Some((t, d, label))
}

fn between(l: &str) -> Option<Answer> {
    let (a, b) = if let Some(m) = re!(r"^(?:time|years|how long|how many years|how much time) (?:between|from) (.+?) (?:and|to|until) (.+)$").captures(l) {
        (m[1].to_string(), m[2].to_string())
    } else if let Some(m) = re!(r"^(.+?) - (.+)$").captures(l) {
        // "Battle of Culloden start date - War of the Roses start date"
        (m[2].to_string(), m[1].to_string())
    } else {
        return None;
    };
    let (_, da, la) = endpoint(&a)?;
    let (_, db, lb) = endpoint(&b)?;
    let (da, db) = (da?, db?);
    let (first, second, f_l, s_l) = if (da.y, da.m, da.d) <= (db.y, db.m, db.d) { (da, db, la.clone(), lb.clone()) } else { (db, da, lb.clone(), la.clone()) };
    Some(Answer::new("time span", vec![
        input(format!("time from the {f_l} to the {s_l}")),
        primary(pod("Result", "Time span", text(span(first, second)))),
        pod("Dates", "Dates", table(None, vec![
            vec![text(f_l), text(show(first))],
            vec![text(s_l), text(show(second))],
        ])),
    ]))
}

fn lists(l: &str) -> Option<Answer> {
    // "random military conflict": a battle or war picked from the table
    if re!(r"^(?:a |show me a |give me a )?random (?:military conflict|battle|war|armed conflict|conflict)$").is_match(l) {
        let n = battles().len();
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.subsec_nanos() as usize ^ d.as_secs() as usize).unwrap_or(0);
        let name = battles()[seed % n][0].split(',').next()?;
        let mut a = card(&find(name, false)?)?;
        a.assumptions.push("a random military conflict from the built-in table".into());
        return Some(a);
    }
    if let Some(m) = re!(r"^(?:battles of (?:the )?(.+)|(.+?) battles)$").captures(l) {
        let war = m.get(1).or(m.get(2))?.as_str();
        let war = match war { "us civil war" | "american civil war" | "civil war" | "u.s. civil war" => "american civil war", w => w };
        let mut rows: Vec<&Vec<&str>> = battles().iter().filter(|r| r[1].to_lowercase().contains(war)).collect();
        if rows.is_empty() {
            return None;
        }
        rows.sort_by_key(|r| parse(r[2], false).map(|d| (d.y, d.m, d.d)));
        let t: Vec<Vec<Content>> = rows.iter().map(|r| vec![text(title(r[0].split(',').next().unwrap_or(""))), text(show(parse(r[2], false).unwrap())), text(r[4]), text(r[9].split(';').next().unwrap_or(""))]).collect();
        return Some(Answer::new("battles", vec![
            input(format!("battles of the {}", title(war))),
            primary(pod("Result", "Battles", text(rows.iter().map(|r| title(r[0].split(',').next().unwrap_or(""))).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Battles", table(Some(vec!["battle", "date", "location", "result"]), t)),
            pod("Note", "Note", text("major battles only")),
        ]));
    }
    if re!(r"^(?:earliest|oldest|first) (?:recorded )?(?:military conflicts|battles|wars)$").is_match(l) {
        let mut rows: Vec<&Vec<&str>> = battles().iter().collect();
        rows.sort_by_key(|r| parse(r[2], false).map(|d| (d.y, d.m, d.d)));
        let t: Vec<Vec<Content>> = rows.iter().take(5).map(|r| vec![text(title(r[0].split(',').next().unwrap_or(""))), text(show(parse(r[2], false).unwrap())), text(r[4])]).collect();
        return Some(Answer::new("battles", vec![
            input("earliest military conflicts"),
            primary(pod("Result", "Earliest", text(rows.iter().take(5).map(|r| title(r[0].split(',').next().unwrap_or(""))).collect::<Vec<_>>().join(" | ")))),
            pod("Table", "Conflicts", table(Some(vec!["conflict", "date", "location"]), t)),
            pod("Note", "Note", text("the earliest recorded war is usually taken to be the conflict between the Sumerian city-states of Lagash and Umma (c. 2500 BCE)")),
        ]));
    }
    None
}

fn property(l: &str) -> Option<Answer> {
    let rx = re!(concat!(r"^(?:", r"(demonym|nationality name|nationality|capital|current countries|present-day countries|modern countries|location|region|key people|important people|famous people|people|commanders|belligerents|sides|outcome|result|winner|casualties|deaths|losses|total amount of material losses|material losses|artillery destroyed|tanks destroyed|equipment destroyed|aircraft lost|ships lost|strength|military forces|number of soldiers|soldiers|beginning|start|end) (?:for|of|in|during|from|involved in|at) (?:the )?(.+)",
        r"|(.+?) (demonym|current countries|present-day countries|modern countries|capital|commanders|casualties|outcome|result|location|elephantry|elephants|cavalry)",
        r"|which (?:current|modern|present-day) countries were (?:ruled|part of|controlled|occupied) by (?:the )?(.+)",
        r"|where did (?:the )?(.+?) take place|where was (?:the )?(.+)",
        r"|who (?:fought|was involved|were the commanders|commanded|won) (?:in |at |during )?(?:the )?(.+)",
        r"|(?:how many )?(?:soldiers|people|troops|men)(?: were)? (?:killed|died) (?:in|at|during) (?:the )?(.+)",
        r"|(?:when did|when was) (?:the )?(.+?) (end|start|begin|founded|fall|collapse)",
        r"|how long did (?:the )?(.+?) last",
        r"|(?:when|what year) (?:was|were|did) (?:the )?(.+?)(?: happen| take place| occur)?",
        r"|(?:the )?(.+?) in (-?\d{1,4})(?: (bce|bc|ce|ad))?",
        r")$"));
    let m = rx.captures(l)?;
    let (key, name): (String, String) = if let Some(k) = m.get(1) { (k.as_str().into(), m[2].into()) }
        else if let Some(n) = m.get(3) { (m[4].into(), n.as_str().into()) }
        else if let Some(n) = m.get(5) { ("current countries".into(), n.as_str().into()) }
        else if let Some(n) = m.get(6).or(m.get(7)) { ("location".into(), n.as_str().into()) }
        else if let Some(n) = m.get(8) {
            let k = if l.starts_with("who won") { "outcome" } else if l.contains("commander") || l.starts_with("who commanded") { "commanders" } else if l.contains("involved") { "people" } else { "belligerents" };
            (k.into(), n.as_str().into())
        }
        else if let Some(n) = m.get(9) { ("casualties".into(), n.as_str().into()) }
        else if let Some(n) = m.get(10) { (match &m[11] { "start" | "begin" | "founded" => "start", _ => "end" }.into(), n.as_str().into()) }
        else if let Some(n) = m.get(12) { ("duration".into(), n.as_str().into()) }
        else if let Some(n) = m.get(13) { ("date".into(), n.as_str().into()) }
        else if let Some(n) = m.get(14) {
            let y: i32 = m[15].parse().ok()?;
            let y = if matches!(m.get(16).map(|x| x.as_str()), Some("bce" | "bc")) || y < 0 { 1 - y.abs() } else { y };
            return at_year(n.as_str(), y);
        }
        else { return None };
    let t = find(&name, false)?;
    let pick = |r: &Vec<&str>, i: usize| (!r[i].is_empty() && r[i] != "—").then(|| r[i].to_string());
    let (label, value) = match (&t, key.as_str()) {
        (Thing::Era(r), "demonym" | "nationality name" | "nationality") => ("demonym", pick(r, 6)?),
        (Thing::Era(r), "capital") => ("capital", pick(r, 5)?),
        (Thing::Era(r), "current countries" | "present-day countries" | "modern countries") => ("present-day countries", pick(r, 7)?),
        (Thing::Era(r), "location" | "region") => ("region", pick(r, 4)?),
        (Thing::Era(r), "key people" | "important people" | "famous people" | "people") => ("key people", pick(r, 8)?),
        (Thing::Battle(r), "location" | "region") => ("location", pick(r, 4)?),
        (Thing::Battle(r), "commanders") => ("commanders", pick(r, 6)?),
        (Thing::Battle(r), "belligerents" | "sides" | "people") => ("belligerents", pick(r, 5)?),
        (Thing::Battle(r), "outcome" | "result" | "winner") => ("result", pick(r, 9)?),
        (Thing::Battle(r), "casualties" | "deaths" | "losses" | "material losses" | "total amount of material losses" | "artillery destroyed" | "tanks destroyed" | "equipment destroyed" | "aircraft lost" | "ships lost") => ("casualties and losses", pick(r, 8)?),
        (Thing::Battle(r), "elephantry" | "elephants" | "cavalry") => ("strength", pick(r, 7)?),
        (_, "beginning") => ("start", show(t.start()?)),
        (Thing::Extra(r), "date") => ("date", if r[1] == r[2] { show(t.start()?) } else { format!("{} to {}", show(t.start()?), show(t.end()?)) }),
        (Thing::Era(_) | Thing::Battle(_), "date") => ("dates", format!("{} to {}", show(t.start()?), show(t.end()?))),
        (Thing::Battle(r), "strength" | "military forces" | "number of soldiers" | "soldiers") => ("strength", pick(r, 7)?),
        (_, "start") => ("start", show(t.start()?)),
        (_, "end") => ("end", show(t.end()?)),
        (_, "duration") => ("duration", span(t.start()?, t.end()?)),
        _ => return None,
    };
    let mut pods = vec![input(format!("{} | {label}", t.name())), primary(pod("Result", &capital(label), text(value)))];
    if let Some(c) = card(&t) {
        pods.extend(c.pods.into_iter().filter(|p| p.id == "Properties"));
    }
    Some(Answer::new("history", pods))
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

/// "Ottoman Empire in 1566", "Minoan civilization in 1600 BCE"
fn at_year(name: &str, y: i32) -> Option<Answer> {
    let t = find(name, false)?;
    let Thing::Era(r) = &t else { return None };
    let (s, e) = (t.start()?, t.end()?);
    let status = if y < s.y {
        format!("did not yet exist (founded {})", show(s))
    } else if y > e.y {
        format!("no longer existed (ended {})", show(e))
    } else {
        let age = y - s.y;
        format!("existed: {}{age} years after its start", if s.approx { "about " } else { "" })
    };
    let moment = r[10].split("; ").find_map(|mm| {
        let (k, v) = mm.split_once(": ")?;
        let ky = parse(k, false)?.y;
        (ky == y).then(|| v.to_string())
    });
    let mut pods = vec![
        input(format!("{} | {}", t.name(), year_text(y))),
        primary(pod("Result", "Status", text(match &moment { Some(mm) => format!("{status}; {mm}"), None => status }))),
    ];
    pods.push(pod("Properties", "Basic information", table(None, era_rows(r))));
    Some(Answer::new("history", pods))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).to_string();
    let l = re!(r"\s+").replace_all(&l, " ").replace('’', "'");
    let l = re!(r"^(?:what (?:is|was|were) |tell me about |show |the )").replace(&l, "").to_string();
    let l = l.trim_start_matches("compare the timelines of ").trim_start_matches("compare timelines of ").trim_start_matches("timeline of ").trim_start_matches("compare ");
    if let Some(a) = timeline(l).or_else(|| lists(l)).or_else(|| between(l)).or_else(|| property(l)) {
        return Some(a);
    }
    let names = split(l);
    let things: Vec<Thing> = names.iter().map(|n| find(n, names.len() > 1)).collect::<Option<Vec<_>>>()?;
    if things.len() == 1 {
        return card(&things[0]);
    }
    // timeline comparison
    let mut header = vec![""];
    let tn: Vec<String> = things.iter().map(|t| t.name()).collect();
    header.extend(tn.iter().map(|s| s.as_str()));
    let row = |lab: &str, f: &dyn Fn(&Thing) -> String| -> Vec<Content> {
        let mut v = vec![text(lab)];
        v.extend(things.iter().map(|t| text(f(t))));
        v
    };
    let mut rows = vec![
        row("start", &|t| t.start().map(show).unwrap_or_default()),
        row("end", &|t| t.end().map(show).unwrap_or_default()),
        row("duration", &|t| match (t.start(), t.end()) { (Some(a), Some(b)) => span(a, b), _ => String::new() }),
    ];
    if things.iter().all(|t| matches!(t, Thing::Era(_))) {
        for (i, lab) in [(4, "region"), (5, "capital"), (6, "demonym")] {
            rows.push(row(lab, &|t| match t { Thing::Era(r) => r[i].to_string(), _ => String::new() }));
        }
    }
    let mut order: Vec<&Thing> = things.iter().collect();
    order.sort_by_key(|t| t.start().map(|d| (d.y, d.m, d.d)).unwrap_or((i32::MAX, 0, 0)));
    Some(Answer::new("history", vec![
        input(format!("{} | timeline", tn.join(", "))),
        primary(pod("Result", "Comparison", table(Some(header), rows))),
        pod("Order", "Chronological order", text(order.iter().map(|t| t.name()).collect::<Vec<_>>().join(" → "))),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_parses() {
        for r in eras() {
            assert_eq!(r.len(), 11, "{r:?}");
            assert!(parse(r[2], false).is_some() && parse(r[3], false).is_some(), "{r:?}");
        }
        for r in battles() {
            assert_eq!(r.len(), 10, "{r:?}");
            assert!(parse(r[2], false).is_some() && parse(r[3], false).is_some(), "{r:?}");
        }
        // 48 BCE is astronomical −47
        assert_eq!(parse("-48-08-09", false).unwrap().y, -47);
        assert_eq!(span(parse("-1274", false).unwrap(), parse("-31", false).unwrap()), "about 1,243 years");
    }
}
