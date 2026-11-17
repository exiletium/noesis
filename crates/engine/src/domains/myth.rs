//! Mythology: gods, heroes and creatures of the Greek, Roman, Egyptian,
//! Hindu, Norse and Mesopotamian traditions, with their domains, symbols,
//! family, equivalents in other pantheons, places, appearance and sources.

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Figure {
    names: Vec<String>,
    culture: String,
    kind: String,
    gender: String,
    domain: String,
    symbols: String,
    family: String,
    equivalent: String,
    other_names: String,
    places: String,
    appearance: String,
    powers: String,
    texts: String,
}

impl Figure {
    fn name(&self) -> &str {
        &self.names[0]
    }
}

// names | culture | kind | gender | domain (patronage) | symbols | family | equivalent | other names | places | appearance | powers | texts
const FIGURES: &str = "\
Zeus|Greek|god, king of the Olympians|male|sky, thunder, lightning, law, order, justice, hospitality|thunderbolt, eagle, bull, oak tree, scepter, aegis|son of Cronus and Rhea; husband of Hera|Jupiter (Roman)|Zeus Olympios, Zeus Xenios, the Cloud-Gatherer|born in a cave on Mount Ida or Mount Dikte, Crete; ruled from Mount Olympus; great sanctuary at Olympia; oracle at Dodona|a powerful bearded man holding a thunderbolt; also took the forms of a bull, a swan, an eagle and golden rain|wields the thunderbolt; controls weather; shape-shifting|Hesiod's Theogony; Homer's Iliad and Odyssey; Homeric Hymns
Hera|Greek|goddess, queen of the Olympians|female|marriage, women, childbirth, family|peacock, cow, pomegranate, diadem, lotus scepter|daughter of Cronus and Rhea; wife and sister of Zeus|Juno (Roman)|Argive Hera, cow-eyed Hera|Argos, Samos (Heraion); Mount Olympus|a regal woman wearing a crown (polos) and holding a scepter|protects marriages; punishes infidelity|Iliad; Theogony
Poseidon|Greek|god|male|sea, earthquakes, storms, horses|trident, horse, bull, dolphin|son of Cronus and Rhea; husband of Amphitrite|Neptune (Roman)|Earth-Shaker, Enosichthon|Isthmus of Corinth; Cape Sounion temple; Atlantis (in Plato)|a bearded man with a trident, often in a chariot drawn by hippocamps|raises storms and earthquakes; created the horse|Odyssey; Iliad
Hades|Greek|god|male|the underworld, the dead, riches of the earth|helm of darkness (cap of invisibility), bident, Cerberus, cypress, key|son of Cronus and Rhea; husband of Persephone|Pluto, Dis Pater, Orcus (Roman)|Plouton, Aidoneus|the underworld; Necromanteion of Ephyra|a stern bearded man enthroned, with Cerberus at his side|rules the dead; invisibility through his helm|Theogony; Homeric Hymn to Demeter
Athena|Greek|goddess|female|wisdom, warfare strategy, crafts, weaving|owl, olive tree, aegis with the Gorgon's head, spear, helmet|daughter of Zeus and Metis, born fully armed from Zeus's head|Minerva (Roman)|Pallas Athena, Athena Parthenos, Glaukopis (grey-eyed)|Athens (Parthenon on the Acropolis)|a woman in armor with helmet, spear and shield|strategic warfare; wisdom; crafts|Iliad; Odyssey; Aeschylus's Eumenides
Apollo|Greek|god|male|sun and light, music, poetry, prophecy, healing, archery|lyre, laurel wreath, bow and arrows, raven, swan|son of Zeus and Leto; twin of Artemis|Apollo (Roman)|Phoebus, Loxias, Apollo Pythios|born on the island of Delos; oracle at Delphi|a beardless young man with a lyre or bow and a laurel wreath|prophecy; music; healing and plague|Homeric Hymn to Apollo; Iliad
Artemis|Greek|goddess|female|the hunt, wilderness, wild animals, the Moon, childbirth, chastity|bow and arrows, deer, hunting dogs, crescent moon, cypress|daughter of Zeus and Leto; twin of Apollo|Diana (Roman)|Phoebe, Cynthia, Potnia Theron (Mistress of Animals)|born on Delos (or Ortygia); Temple of Artemis at Ephesus|a young huntress with a bow, quiver and short tunic, often with a deer|unerring archery; protects young women and animals|Homeric Hymns; Callimachus's Hymn to Artemis
Ares|Greek|god|male|war, violence, courage|spear, helmet, shield, dog, vulture, boar|son of Zeus and Hera|Mars (Roman)|Enyalios|Thrace; the Areopagus in Athens|an armed warrior with helmet and spear|battle-lust and courage in war|Iliad
Aphrodite|Greek|goddess|female|love, beauty, desire, pleasure, fertility|dove, swan, rose, myrtle, scallop shell, mirror, apple|born from sea foam near Cyprus (Hesiod) or daughter of Zeus and Dione (Homer); wife of Hephaestus|Venus (Roman)|Cypris, Cytherea, Aphrodite Urania|Paphos on Cyprus; Cythera; Corinth|a beautiful woman, often nude or rising from the sea on a shell|inspires desire; her magic girdle makes anyone fall in love|Theogony; Homeric Hymns; Iliad
Hephaestus|Greek|god|male|fire, the forge, metalworking, craftsmen|hammer, anvil, tongs, forge|son of Hera (alone, or with Zeus); husband of Aphrodite|Vulcan (Roman)|Hephaistos, Mulciber|forge on Mount Olympus or under Mount Etna; Lemnos; Hephaisteion in Athens|a bearded smith, lame, with hammer and tongs|master craftsman: made Achilles's armor and Zeus's thunderbolts|Iliad; Odyssey
Hermes|Greek|god, messenger of the gods|male|travelers, trade, thieves, messengers, boundaries, guide of souls|caduceus, winged sandals, winged helmet (petasos), tortoise|son of Zeus and Maia|Mercury (Roman)|Psychopomp, Argeiphontes|born on Mount Cyllene, Arcadia|a youth in a winged hat and sandals, carrying the caduceus|speed; cunning; leads souls to the underworld|Homeric Hymn to Hermes; Odyssey
Demeter|Greek|goddess|female|agriculture, grain, harvest, fertility|sheaf of wheat, torch, cornucopia, poppy|daughter of Cronus and Rhea; mother of Persephone|Ceres (Roman)|Deo, Thesmophoros|Eleusis (Eleusinian Mysteries)|a mature woman holding sheaves of grain and a torch|controls the growth of crops and the seasons|Homeric Hymn to Demeter
Dionysus|Greek|god|male|wine, festivity, theatre, ecstasy, fertility|thyrsus, grapevine, ivy, wine cup (kantharos), leopard, panther|son of Zeus and Semele|Bacchus, Liber (Roman)|Bacchus, Bromios, Iacchus|born from Zeus's thigh on Mount Nysa; Theatre of Dionysus in Athens; Delphi|a young or bearded man crowned with ivy, holding a thyrsus|induces ecstatic madness; turns water to wine|Euripides's Bacchae; Homeric Hymns
Hestia|Greek|goddess|female|the hearth, home, family, the state|hearth fire, kettle|first-born daughter of Cronus and Rhea|Vesta (Roman)||worshipped at every household hearth and the prytaneion (town hall) of each city; few temples of her own — her main cult was the public hearth, as at Delphi and Olympia|a modest veiled woman|keeper of the sacred flame|Theogony; Homeric Hymns
Persephone|Greek|goddess, queen of the underworld|female|spring, vegetation, the underworld|pomegranate, sheaf of grain, torch, flowers|daughter of Zeus and Demeter; wife of Hades|Proserpina (Roman)|Kore (the Maiden), Despoina|Eleusis; the underworld|a young woman holding a pomegranate or torch|her return from the underworld each year brings spring|Homeric Hymn to Demeter; Ovid's Metamorphoses
Nike|Greek|goddess|female|victory|wings, laurel wreath, palm branch|daughter of the Titan Pallas and Styx|Victoria (Roman)|Winged Victory|Temple of Athena Nike on the Acropolis; Samothrace|a winged woman, as in the Winged Victory of Samothrace|grants victory; flies over battlefields|Theogony
Prometheus|Greek|Titan|male|forethought, fire, craft; champion of humankind|fire, fennel stalk, eagle (his punishment)|son of the Titan Iapetus and Clymene (or Themis); brother of Atlas and Epimetheus||the Firebringer|chained to a rock in the Caucasus Mountains|a Titan chained to a rock while an eagle eats his liver|stole fire from the gods for humans; foresight|Hesiod's Theogony and Works and Days; Aeschylus's Prometheus Bound; Shelley's Prometheus Unbound; Mary Shelley's Frankenstein, or The Modern Prometheus; Goethe's poem Prometheus
Heracles|Greek|hero, demigod|male|strength, heroes, athletes|club, lion skin (Nemean lion), bow and arrows|son of Zeus and Alcmene|Hercules (Roman)|Alcides|born in Thebes; Tiryns; Mount Oeta (his death)|a muscular bearded man with a club and lion skin|superhuman strength; completed the Twelve Labors|Sophocles's Women of Trachis; Euripides's Heracles; Apollodorus's Library
Hercules|Roman|hero, demigod|male|strength, heroes|club, lion skin, bow|son of Jupiter and Alcmene|Heracles (Greek)|Alcides|Ara Maxima in Rome|a muscular bearded man with a club and lion skin|superhuman strength; the Twelve Labors|Virgil's Aeneid; Seneca's Hercules Furens
Theseus|Greek (Athenian)|hero|male|founder-hero of Athens|sword, sandals|son of Aegeus (or Poseidon) and Aethra||the founder-king of Athens|born in Troezen; Athens; Crete (the Labyrinth)|a young warrior with a sword or club|slew the Minotaur; unified Attica|Plutarch's Life of Theseus; Bacchylides; Euripides
Pegasus|Greek|creature|male|poetic inspiration; carries Zeus's thunderbolts|wings, the Hippocrene spring|sprang from the blood of Medusa, fathered by Poseidon||the winged horse|Mount Helicon (Hippocrene spring); Corinth (tamed by Bellerophon at the Pirene spring); Mount Olympus; the constellation Pegasus|a white winged horse|flight; his hoof struck springs from the ground|Hesiod's Theogony; Pindar
Medusa|Greek|creature, Gorgon|female|protection (her head on the aegis)|snakes for hair, Gorgoneion|daughter of Phorcys and Ceto|||lived at the edge of the world; slain by Perseus|a woman with snakes for hair whose gaze turns people to stone|petrifying gaze|Theogony; Ovid's Metamorphoses
Jupiter|Roman|god, king of the gods|male|sky, thunder, the state, law|eagle, thunderbolt, oak, scepter|son of Saturn and Ops; husband of Juno|Zeus (Greek)|Jove, Jupiter Optimus Maximus, Jupiter Capitolinus, Jupiter Tonans (the Thunderer), Diespiter|Temple of Jupiter Optimus Maximus on the Capitoline Hill, Rome|a bearded man with thunderbolt and eagle|controls the weather; guarantees oaths and treaties|Virgil's Aeneid; Ovid's Metamorphoses
Juno|Roman|goddess, queen of the gods|female|marriage, childbirth, women, the Roman state|peacock, pomegranate, diadem|daughter of Saturn and Ops; wife of Jupiter|Hera (Greek)|Juno Regina, Juno Moneta, Juno Lucina|Capitoline Temple (with Jupiter and Minerva)|a matronly woman in armor or a goat-skin cloak|protects women, marriage, childbirth and the finances of the state (Juno Moneta)|Aeneid
Neptune|Roman|god|male|the sea, fresh water, horses|trident, dolphin, horse|son of Saturn and Ops|Poseidon (Greek)||Rome (Circus Flaminius)|a bearded man with a trident|rules the seas|Aeneid
Minerva|Roman|goddess|female|wisdom, crafts, strategic warfare|owl, olive branch|daughter of Jupiter|Athena (Greek)||Capitoline Triad temple, Rome|a woman with helmet, spear and shield|wisdom; crafts|Ovid's Metamorphoses
Mars|Roman|god|male|war, agriculture (early), protector of Rome|spear, shield, wolf, woodpecker|son of Jupiter and Juno; father of Romulus and Remus|Ares (Greek)|Mars Gradivus, Mars Ultor|Campus Martius and Temple of Mars Ultor, Rome|an armed warrior in a helmet|war; guardian of Rome|Aeneid; Ovid's Fasti
Venus|Roman|goddess|female|love, beauty, desire, fertility, victory|dove, myrtle, rose, scallop shell|daughter of Jupiter and Dione (or born of the sea); mother of Aeneas and Cupid|Aphrodite (Greek)|Venus Genetrix, Venus Victrix|Temple of Venus Genetrix, Rome; Eryx in Sicily|a beautiful woman, often with Cupid|inspires love|Aeneid; Lucretius
Vulcan|Roman|god|male|fire, volcanoes, metalworking, the forge|hammer, tongs, anvil|son of Jupiter and Juno|Hephaestus (Greek)|Mulciber|Volcanal in the Roman Forum; forge under Mount Etna|a bearded smith with hammer and tongs, in a workman's cap; also manifests as fire and volcanic flame|forges weapons for gods and heroes|Aeneid book 8
Mercury|Roman|god|male|commerce, messengers, travelers, thieves|caduceus, winged sandals and hat, purse|son of Jupiter and Maia|Hermes (Greek)||Circus Maximus temple, Rome|a youth with winged hat and caduceus|speed; guide of souls|Aeneid
Diana|Roman|goddess|female|the hunt, the Moon, wild animals, childbirth|bow and quiver, deer, hunting dogs, crescent moon|daughter of Jupiter and Latona; twin of Apollo|Artemis (Greek)|Diana Nemorensis, Trivia|sanctuary at Lake Nemi; Aventine Hill temple|a huntress with a bow and a crescent moon|protects women in childbirth and the wild|Ovid's Metamorphoses
Ceres|Roman|goddess|female|agriculture, grain, fertility|wheat sheaf, sickle, torch|daughter of Saturn and Ops|Demeter (Greek)||Aventine Temple of Ceres, Rome|a woman with a wreath of grain|growth of crops|Ovid's Fasti
Bacchus|Roman|god|male|wine, revelry|grapevine, thyrsus|son of Jupiter and Semele|Dionysus (Greek)|Liber|Rome|a young man crowned with vine leaves|ecstasy|Ovid's Metamorphoses
Vesta|Roman|goddess|female|the hearth, home, the state|sacred fire|daughter of Saturn and Ops|Hestia (Greek)||Temple of Vesta in the Roman Forum, tended by the Vestal Virgins|rarely shown; a veiled woman|eternal flame of Rome|Ovid's Fasti
Saturn|Roman|god|male|agriculture, wealth, time, the Golden Age|sickle or scythe|son of Caelus and Terra; father of Jupiter, Juno, Neptune, Pluto|Cronus (Greek)|Saturnus|Temple of Saturn in the Roman Forum (the state treasury); festival of Saturnalia|an old bearded man with a sickle|ruled during the Golden Age|Virgil's Georgics and Aeneid; Macrobius's Saturnalia
Janus|Roman|god|male|beginnings, gates, doorways, transitions, time|two faces, keys, staff|son of Caelus|none|Janus Bifrons|Temple of Janus in the Roman Forum (doors open in wartime)|a head with two faces looking in opposite directions|sees the past and the future|Ovid's Fasti
Fortuna|Roman|goddess|female|luck, fortune, fate|wheel of fortune, cornucopia, rudder, globe|daughter of Jupiter (in some accounts)|Tyche (Greek)|Fortuna Primigenia|Praeneste (Palestrina) sanctuary|a woman with a rudder, cornucopia and wheel, sometimes blindfolded|bestows good or bad luck|Boethius's Consolation of Philosophy
Orcus|Roman|god|male|the underworld, punisher of broken oaths|none distinctive|identified with Dis Pater and Pluto|Hades (Greek)|Dis Pater, Pluto|the underworld|a hairy, bearded giant, as in Etruscan tomb paintings|punishes oath-breakers|Etruscan tomb art (Tomb of Orcus, Tarquinia)
Cupid|Roman|god|male|desire, erotic love|bow and arrows, wings, torch|son of Venus and Mars|Eros (Greek)|Amor|Rome|a winged child or youth with a bow|his arrows make people fall in love|Apuleius's The Golden Ass (Cupid and Psyche)
Ra|Egyptian|god|male|the Sun, creation, kingship|sun disk with a cobra (uraeus), falcon, ankh, solar barque|self-created; father of Shu and Tefnut|Helios (Greek, loosely)|Re, Amun-Ra, Atum-Ra, Ra-Horakhty|Heliopolis (Iunu)|a man with a falcon's head topped by a sun disk|sails the sun across the sky by day and through the underworld by night|Pyramid Texts; Book of the Dead
Amun-Ra|Egyptian|god, king of the gods|male|the Sun, creation, kingship, hidden power|ram, goose, double-plumed crown, sun disk, ankh|merger of Amun and Ra; husband of Mut; father of Khonsu|Zeus (Greek), Jupiter Ammon (Roman)|Amun, Amon, Ammon, Amen|Karnak and Luxor temples, Thebes; Siwa Oasis oracle|a man with a tall double-plumed crown, or ram-headed|creator of all things; hidden (invisible) power; grants kingship and victory|Hymns to Amun; Great Harris Papyrus
Osiris|Egyptian|god|male|the afterlife, the dead, resurrection, fertility|crook and flail, atef crown, djed pillar|son of Geb and Nut; husband of Isis; father of Horus|Hades / Dionysus (Greek, loosely)|Wennefer, Lord of the Dead|Abydos|a mummified man with green or black skin wearing the atef crown, holding crook and flail|judges the dead; rebirth|Pyramid Texts; Book of the Dead; Plutarch's On Isis and Osiris
Isis|Egyptian|goddess|female|magic, motherhood, healing, protection|throne-shaped headdress, ankh, sistrum, wings, tyet knot|daughter of Geb and Nut; wife of Osiris; mother of Horus|Demeter (Greek, loosely)|Aset, Mother of God, Lady of Ten Thousand Names|Philae; Behbeit el-Hagar; worshipped throughout the Roman Empire|a woman with a throne-shaped headdress or cow horns with a sun disk|powerful magic; resurrected Osiris|Pyramid Texts; Apuleius's The Golden Ass
Horus|Egyptian|god|male|the sky, kingship, protection|Eye of Horus (wedjat), falcon, double crown (pschent)|son of Osiris and Isis|Apollo (Greek)|Har, Hor, Heru, Harsiese, Harpocrates (Horus the Child), Ra-Horakhty, Horus the Elder (Haroeris)|Edfu; Hierakonpolis (Nekhen)|a man with a falcon's head, or a falcon|protector of the pharaoh; avenged his father against Set|Contendings of Horus and Set; Pyramid Texts
Anubis|Egyptian|god|male|embalming, mummification, the dead, cemeteries|jackal, flail, embalming tools, imiut fetish|son of Osiris and Nephthys (or of Ra)|Hermanubis (Greek, merged with Hermes)|Inpu, Anpu|Cynopolis; necropolises such as Saqqara|a man with a black jackal's head, or a lying black jackal|weighs hearts against the feather of Ma'at; guides souls|Book of the Dead; Pyramid Texts
Bastet|Egyptian|goddess|female|home, fertility, cats, protection, joy|cat, sistrum, ointment jar, lioness (early form)|daughter of Ra|Artemis (Greek, loosely)|Bast, Ubaste|Bubastis|a woman with a cat's head, or a cat|protects homes from evil spirits and disease|Pyramid Texts
Thoth|Egyptian|god|male|writing, wisdom, the Moon, magic, science, judgment|ibis, baboon, papyrus scroll, reed pen, crescent moon|self-created, or son of Ra|Hermes (Greek), as Hermes Trismegistus|Djehuty, Tehuti|Hermopolis (Khemenu)|a man with an ibis head, or a baboon|invented writing; records the judgment of the dead|Book of the Dead; Hermetic texts
Sobek|Egyptian|god|male|the Nile, crocodiles, fertility, military prowess|crocodile, sun disk with plumes|son of Neith|Souchos (Greek)|Sebek, Suchos|Kom Ombo; Crocodilopolis (Faiyum)|a man with a crocodile's head, or a crocodile, often with a sun disk and plumes|protects against dangers of the Nile; makes the land fertile|Pyramid Texts
Hathor|Egyptian|goddess|female|love, music, dance, joy, motherhood, foreign lands, mining, women|cow horns with a sun disk, sistrum, menat necklace, mirror|daughter of Ra; consort of Horus|Aphrodite (Greek)|Mistress of the West, Lady of Turquoise|Dendera|a woman with cow horns and a sun disk, or a cow|patron of music, dance, women and miners; welcomes the dead|Pyramid Texts; love poetry
Set|Egyptian|god|male|chaos, storms, deserts, violence, foreigners|was scepter, Set animal|son of Geb and Nut; brother and murderer of Osiris|Typhon (Greek)|Seth, Sutekh|Ombos (Naqada); Avaris|a man with the head of the unidentified Set animal|storms; defends Ra's barque against the serpent Apep|Contendings of Horus and Set
Ptah|Egyptian|god|male|craftsmen, architects, creation|was scepter, djed pillar, ankh|husband of Sekhmet|Hephaestus (Greek)||Memphis|a mummified man with a skullcap and straight beard|created the world by thought and word|Memphite Theology (Shabaka Stone)
Egyptian sphinx|Egyptian|creature|male|royal power, guardian|lion's body, human head with royal headdress (nemes)||||the Great Sphinx of Giza; avenues of sphinxes at Karnak|a lying lion with the head of a pharaoh, wingless|guardian of temples and tombs; benevolent|
Assyrian sphinx;lamassu;shedu|Assyrian (Mesopotamian)|creature, protective deity|male|protection of palaces and cities|winged bull or lion body, bearded human head, horned crown||||palace gateways at Nimrud, Khorsabad and Nineveh|a winged bull or lion with a bearded human head and horned cap; usually shown with five legs|wards off evil at gateways|Neo-Assyrian palace inscriptions
Greek sphinx|Greek|creature|female|riddles, destruction|lion's body, woman's head, eagle's wings|daughter of Typhon and Echidna|||Thebes|a winged lion with a woman's head|devoured those who failed her riddle, solved by Oedipus|Sophocles's Oedipus Rex; Hesiod's Theogony
Brahma|Hindu|god, the Creator|male|creation, knowledge, the Vedas|lotus, the Vedas, water pot (kamandalu), prayer beads, swan (hamsa)|born from a lotus from Vishnu's navel; consort Saraswati||Prajapati, Pitamaha, Svayambhu|Pushkar temple, Rajasthan (one of very few dedicated to him)|four faces and four arms, red-skinned, seated on a lotus or swan|creates the universe and all beings|Vedas; Puranas (Brahma Purana); Mahabharata
Vishnu|Hindu|god, the Preserver|male|preservation, protection, dharma|conch (shankha), discus (Sudarshana chakra), mace (gada), lotus; Garuda|consort Lakshmi||Narayana, Hari, and ten avatars (Dashavatara) including Rama and Krishna|Vaikuntha; Tirupati; Badrinath; Srirangam|blue-skinned with four arms, reclining on the serpent Shesha|descends as avatars to restore cosmic order|Vedas; Vishnu Purana; Bhagavad Gita
Shiva|Hindu|god, the Destroyer|male|destruction and transformation, yoga, meditation, the arts|trident (trishula), third eye, crescent moon, serpent, damaru drum, lingam; Nandi the bull|consort Parvati; father of Ganesha and Kartikeya||Mahadeva, Rudra, Nataraja, Shankara|Mount Kailash; Varanasi; twelve Jyotirlingas|ash-smeared ascetic with matted hair, blue throat, a serpent around his neck|destroys to allow re-creation; cosmic dance|Vedas (as Rudra); Shiva Purana
Ganesha;ganesh|Hindu|god|male|beginnings, wisdom, removing obstacles, arts and sciences|elephant head, broken tusk, modak sweets, axe, noose, mouse vehicle (Mushika)|son of Shiva and Parvati||Ganapati, Vinayaka, Pillaiyar, Vighneshvara|worshipped everywhere; Ashtavinayaka temples in Maharashtra|an elephant-headed, pot-bellied god with four arms; he has 32 classical forms, such as Bala (child), Taruna (youth), Heramba (five-headed, riding a lion) and Ucchishta|removes obstacles; invoked at the start of any undertaking|Ganesha Purana; Mudgala Purana; Ganapati Atharvashirsha
Krishna|Hindu|god, eighth avatar of Vishnu|male|love, compassion, protection, divine joy|flute (bansuri), peacock feather, cows, butter|son of Devaki and Vasudeva, raised by Yashoda and Nanda||Govinda, Gopala, Vasudeva, Keshava|born in Mathura; childhood in Vrindavan; Dwarka|a blue- or dark-skinned youth playing the flute, crowned with a peacock feather, often with Radha or cows|delivered the Bhagavad Gita; lifted Govardhan Hill|Mahabharata (Bhagavad Gita); Bhagavata Purana; Harivamsa
Lakshmi|Hindu|goddess|female|wealth, fortune, prosperity, beauty|lotus, gold coins, elephants, owl|consort of Vishnu||Shri, Padma, Kamala|worshipped at Diwali across India and Nepal; Mahalakshmi Temple, Kolhapur and Mumbai|a woman seated on a lotus with four arms, gold coins flowing from her hand, flanked by elephants|grants prosperity|Shri Sukta (Rigveda appendix); Vishnu Purana
Saraswati|Hindu|goddess|female|knowledge, music, art, speech, learning|veena, book, swan, lotus, prayer beads|consort of Brahma||Sharada, Vagdevi|Basar, Telangana; worshipped at Vasant Panchami|a woman in white playing a veena, seated on a lotus or swan|bestows learning and eloquence|Rigveda; Puranas
Parvati|Hindu|goddess|female|love, fertility, devotion, power|lotus, trident, mirror; lion or tiger|daughter of Himavan; consort of Shiva; mother of Ganesha||Uma, Gauri, Shakti|Himalayas|a fair woman beside Shiva|the gentle form of Shakti|Shiva Purana; Kumarasambhava
Durga|Hindu|goddess|female|protection, war against evil, strength|weapons in many arms, lion or tiger mount, trident|a form of Parvati / Shakti||Mahishasuramardini|Kolkata (Durga Puja)|a ten-armed warrior goddess riding a lion|slew the buffalo demon Mahishasura|Devi Mahatmya (Markandeya Purana)
Kali|Hindu|goddess|female|time, destruction, death, liberation|severed heads, sword, skull garland, protruding tongue|a fierce form of Durga / Parvati||Kalika, Mahakali|Kalighat, Kolkata; Dakshineswar|a dark-skinned goddess with a garland of skulls and protruding tongue|destroys evil and ego|Devi Mahatmya
Hanuman|Hindu|god|male|devotion, strength, courage|mace (gada), mountain, monkey form|son of Vayu (wind god) and Anjana||Anjaneya, Bajrangbali, Maruti|Ayodhya; Hampi (Kishkindha)|a monkey-faced god carrying a mace, often lifting a mountain|immense strength; flight; can change size|Ramayana
Garuda|Hindu|divine bird, mount of Vishnu|male|speed, strength, protection against snakes|eagle, serpents|son of the sage Kashyapa and Vinata||Suparna, Garutman|national emblem of Indonesia and Thailand|a giant eagle-like bird, often with a human body, golden wings and a beak; carries Vishnu|great speed and strength; enemy of serpents|Mahabharata; Garuda Purana
Annapurna|Hindu|goddess|female|food, nourishment|ladle, bowl of rice or porridge|a form of Parvati||Annapoorna, Annada|Annapurna Devi Mandir, Varanasi|a woman holding a pot of food and a ladle|ensures no one goes hungry|Annada Mangal (Bharatchandra); Shiva Purana; Annapurna Stotram (Adi Shankara); Annapurna Upanishad
Indra|Hindu (Vedic)|god, king of the devas|male|thunder, rain, storms, war|thunderbolt (vajra), white elephant Airavata, rainbow|son of Aditi and Kashyapa||Shakra, Devendra|Svarga (Amaravati)|a warrior on a white elephant holding a vajra|slew the serpent Vritra to release the waters|Rigveda
Rama|Hindu|god, seventh avatar of Vishnu|male|righteousness, virtue, the ideal king|bow and arrows|son of King Dasharatha; husband of Sita||Ramachandra, Raghava|born in Ayodhya|a blue-skinned prince with bow and arrows|defeated Ravana|Ramayana
Ānanda;ananda|Buddhist|disciple of the Buddha|male|attendant of the Buddha||cousin of Gautama Buddha||Dhammabhandagarika (Treasurer of the Dhamma)|Kapilavastu; Rajagaha (First Buddhist Council)|a monk in robes, often shown young, beside the Buddha|his exceptional memory: he remembered every discourse of the Buddha and recited them at the First Council|Pali Canon (Sutta Pitaka)
Odin|Norse|god, chief of the Aesir|male|wisdom, war, death, poetry, magic|ravens Huginn and Muninn, spear Gungnir, eight-legged horse Sleipnir, wolves|son of Borr and Bestla; husband of Frigg; father of Thor||Woden, Wotan, the All-Father|Asgard (Valhalla)|a one-eyed old man with a wide-brimmed hat and spear|wisdom gained by sacrificing an eye; runes|Poetic Edda; Prose Edda
Thor|Norse|god|male|thunder, storms, strength, protection of humankind|hammer Mjölnir, belt of strength, iron gloves, goats|son of Odin and Jörð|Jupiter (Roman, by interpretatio)|Donar, Thunor|Asgard (Thrudheim)|a red-bearded warrior with a hammer|great strength; slays giants|Poetic Edda; Prose Edda
Loki|Norse|god, trickster|male|mischief, trickery, fire|snake, fly, salmon (forms he took)|son of Fárbauti and Laufey||Loptr, Hveðrungr|Asgard|a handsome shape-shifter|shape-shifting; cunning|Poetic Edda; Prose Edda
Freya;freyja|Norse|goddess|female|love, beauty, fertility, war, magic (seiðr)|necklace Brísingamen, falcon-feather cloak, chariot pulled by cats, boar|daughter of Njord||Vanadís|Fólkvangr|a beautiful woman in a cat-drawn chariot|seiðr magic; receives half the battle-slain|Poetic Edda; Prose Edda
";

fn figures() -> &'static Vec<Figure> {
    static F: OnceLock<Vec<Figure>> = OnceLock::new();
    F.get_or_init(|| {
        FIGURES
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.split('|').collect();
                if f.len() < 13 {
                    return None;
                }
                Some(Figure {
                    names: f[0].split(';').map(String::from).collect(),
                    culture: f[1].into(),
                    kind: f[2].into(),
                    gender: f[3].into(),
                    domain: f[4].into(),
                    symbols: f[5].into(),
                    family: f[6].into(),
                    equivalent: f[7].into(),
                    other_names: f[8].into(),
                    places: f[9].into(),
                    appearance: f[10].into(),
                    powers: f[11].into(),
                    texts: f[12].into(),
                })
            })
            .collect()
    })
}

fn find(s: &str) -> Option<&'static Figure> {
    let s = s.trim().trim_end_matches("'s").trim();
    let low = s.to_lowercase();
    if let Some(f) = figures().iter().find(|f| f.names.iter().any(|n| n.to_lowercase() == low)) {
        return Some(f);
    }
    let s = re!(r"^(?:the )?(?:(greek|roman|egyptian|hindu|norse) )?(?:god(?:dess)? |hero |deity |titan )?").replace(s, "").to_string();
    let s = s.trim().trim_end_matches("'s").trim();
    let low = s.to_lowercase();
    figures().iter().find(|f| f.names.iter().any(|n| n.to_lowercase() == low))
}

fn field(f: &Figure, what: &str) -> Option<(&'static str, String)> {
    let v = |s: &str| if s.is_empty() { None } else { Some(s.to_string()) };
    Some(match what {
        "symbols" | "symbol" => ("symbols", v(&f.symbols)?),
        "powers" | "power" | "special ability" | "abilities" | "ability" => ("powers", v(&f.powers)?),
        "domain" | "patronages" | "patronage" | "protect" => ("domain", v(&f.domain)?),
        "gender" => ("gender", v(&f.gender)?),
        "culture" => ("culture", format!("{} mythology", f.culture)),
        "born" | "birthplace" | "places" | "temple" | "worshipped" => ("places", v(&f.places)?),
        "appearance" | "forms" | "iconography" => ("appearance", v(&f.appearance)?),
        "other names" => ("other names", v(&f.other_names)?),
        "texts" => ("texts", v(&f.texts)?),
        "family" | "parents" => ("family", v(&f.family)?),
        "equivalent" => ("equivalent", v(&f.equivalent)?),
        _ => return None,
    })
}

fn props(f: &Figure) -> Content {
    let mut r = vec![("culture", format!("{} mythology", f.culture)), ("type", f.kind.clone()), ("gender", f.gender.clone()), ("domain", f.domain.clone())];
    for (k, v) in [("symbols", &f.symbols), ("family", &f.family), ("equivalent", &f.equivalent), ("other names", &f.other_names), ("places", &f.places), ("appearance", &f.appearance), ("powers", &f.powers), ("sources", &f.texts)] {
        if !v.is_empty() {
            r.push((k, v.clone()));
        }
    }
    table(None, r.into_iter().map(|(k, v)| vec![text(k), text(v)]).collect())
}

fn card(f: &Figure) -> Answer {
    Answer::new("mythology", vec![
        pod("Input", "Input interpretation", text(format!("{} ({} mythology)", f.name(), f.culture))),
        primary(pod("Result", "Description", text(format!("{} {} — {}", f.culture, f.kind, f.domain)))),
        pod("Properties", "Properties", props(f)),
    ])
}

const QWORDS: &str = r"symbols?|powers?|special ability|abilities|ability|patronages?|domain|gender|culture|other names|appearance|body forms|forms|iconography|texts|family|parents|equivalents?|birthplace";

fn norm(w: &str) -> &'static str {
    match w {
        "symbol" | "symbols" => "symbols",
        "power" | "powers" | "special ability" | "abilities" | "ability" => "powers",
        "patronage" | "patronages" | "domain" => "domain",
        "gender" => "gender",
        "culture" => "culture",
        "other names" => "other names",
        "appearance" | "body forms" | "forms" | "iconography" => "appearance",
        "texts" => "texts",
        "family" | "parents" => "family",
        "equivalent" | "equivalents" => "equivalent",
        _ => "places",
    }
}

fn one(f: &Figure, what: &str) -> Option<Answer> {
    let (label, v) = field(f, what)?;
    let mut c = label.chars();
    let title = c.next().map(|x| x.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default();
    Some(Answer::new("mythology", vec![pod("Input", "Input interpretation", text(format!("{} ({} mythology) — {label}", f.name(), f.culture))), primary(pod("Result", &title, text(v))), pod("Properties", "Properties", props(f))]))
}

fn many(fs: &[&Figure], what: Option<&str>) -> Option<Answer> {
    let what = what.unwrap_or("domain");
    let vals: Vec<(String, String)> = fs.iter().map(|f| (f.name().to_string(), field(f, what).map(|x| x.1).unwrap_or_else(|| "—".into()))).collect();
    let main = vals.iter().map(|(n, v)| format!("{n}: {v}")).collect::<Vec<_>>().join(" | ");
    let rowsf: [(&str, fn(&Figure) -> String); 6] = [
        ("culture", |f| f.culture.clone()),
        ("type", |f| f.kind.clone()),
        ("domain", |f| f.domain.clone()),
        ("symbols", |f| f.symbols.clone()),
        ("family", |f| f.family.clone()),
        ("equivalent", |f| f.equivalent.clone()),
    ];
    let mut head = vec![""];
    head.extend(fs.iter().map(|f| f.name()));
    let t = rowsf.iter().map(|(k, g)| {
        let mut r = vec![text(*k)];
        r.extend(fs.iter().map(|f| text(g(f))));
        r
    }).collect();
    Some(Answer::new("mythology", vec![pod("Input", "Input interpretation", text(fs.iter().map(|f| f.name()).collect::<Vec<_>>().join(" vs "))), primary(pod("Result", what, text(main))), pod("Table", "Comparison", table(Some(head), t))]))
}

fn split_names(s: &str) -> Option<Vec<&'static Figure>> {
    let s = re!(r"^(?:the )?(?:goddesses|gods|deities) ").replace(s, "").to_string();
    let parts: Vec<String> = re!(r"\s*(?:,\s*and\s+|,|\band\b|\bvs\.?\b|\bversus\b|&)\s*").split(&s).filter(|x| !x.trim().is_empty()).map(String::from).collect();
    if parts.len() < 2 {
        return None;
    }
    parts.iter().map(|p| find(p)).collect()
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    // questions about one figure
    let pats: [(&Regex, usize, Option<&str>); 16] = [
        (re!(r"^where was (?:the )?(.+?) born$"), 1, Some("places")),
        (re!(r"^what culture (?:does|did|is) (?:the )?(.+?) (?:come|from|come from|belong to)(?: from)?$"), 1, Some("culture")),
        (re!(r"^locations? (?:significant|important|associated) (?:to|with) (?:the )?(.+)$"), 1, Some("places")),
        (re!(r"^where (?:was|is|were) (?:the )?temples? (?:for|of|to) (?:the )?(.+)$"), 1, Some("places")),
        (re!(r"^where (?:is|was|are) (?:the )?(.+?) worshipp?ed$"), 1, Some("places")),
        (re!(r"^(?:show me )?what (?:does|did) (?:the )?(.+?) look like$"), 1, Some("appearance")),
        (re!(r"^(?:show me )?what (?:the )?(.+?) looks? like$"), 1, Some("appearance")),
        (re!(r"^(?:what are )?(?:the )?texts (?:that )?(?:the )?(.+?) (?:is|was|appears) in$"), 1, Some("texts")),
        (re!(r"^what form does (?:the )?(.+?) take$"), 1, Some("appearance")),
        (re!(r"^what (?:are|were) the texts (?:that )?(?:the )?(.+?) (?:is|was|appears) in$"), 1, Some("texts")),
        (re!(r"^(?:literary references|references|sources|texts) (?:for|to|about|of) (?:the )?(.+)$"), 1, Some("texts")),
        (re!(r"^what (?:does|did) (?:the )?(.+?) protect$"), 1, Some("domain")),
        (re!(r"^what else (?:was|is) (?:the )?(.+?) called$"), 1, Some("other names")),
        (re!(r"^(?:other names|names|epithets) (?:of |for )?(?:the )?(.+?)(?: is| was)?(?: known by| called)?$"), 1, Some("other names")),
        (re!(r"^(?:who (?:is|was) |tell me about )(?:the )?(.+)$"), 1, None),
        (re!(r"^what is (?:the )?(.+?)'s gender$"), 1, Some("gender")),
    ];
    for (r, g, what) in pats {
        if let Some(c) = r.captures(l) {
            if let Some(f) = find(&c[g]) {
                return match what {
                    Some(w) => one(f, w),
                    None => Some(card(f)),
                };
            }
        }
    }
    // "compare the symbols of Fortuna and Nike", "the symbols of anubis and bastet", "Zeus symbols", "Amun Ra powers"
    let qw = format!(r"^(?:compare )?(?:the )?({QWORDS}) (?:of|for) (?:the )?(.+)$");
    static QR: OnceLock<Regex> = OnceLock::new();
    let qr = QR.get_or_init(|| Regex::new(&qw).unwrap());
    if let Some(c) = qr.captures(l) {
        let w = norm(&c[1]);
        if let Some(fs) = split_names(&c[2]) {
            return many(&fs, Some(w));
        }
        let f = find(&c[2])?;
        return one(f, w);
    }
    let qw2 = format!(r"^(?:the )?(.+?)'?s? ({QWORDS})$");
    static QR2: OnceLock<Regex> = OnceLock::new();
    let qr2 = QR2.get_or_init(|| Regex::new(&qw2).unwrap());
    if let Some(c) = qr2.captures(l) {
        let name = c[1].replace("amun ra", "amun-ra");
        if let Some(f) = find(&name).or_else(|| find(c[1].trim_end_matches('\''))) {
            return one(f, norm(&c[2]));
        }
    }
    // "goddesses diana vs venus", "Assyrian sphinx vs Egyptian sphinx", "compare A and B"
    let body = l.trim_start_matches("compare ");
    if let Some(fs) = split_names(body) {
        return many(&fs, None);
    }
    // bare names: "Persephone", "the roman god Saturn"
    let f = find(&l.replace("amun ra", "amun-ra"))?;
    // planets and other homonyms keep their usual meaning unless a pantheon is named
    let bare = ["jupiter", "saturn", "mars", "venus", "mercury", "neptune", "pluto", "ceres", "juno", "vesta", "diana", "nike", "pegasus", "janus", "set", "ra", "rama", "kali", "isis", "ananda"];
    if bare.contains(&l) {
        return None;
    }
    Some(card(f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_parse() {
        assert!(figures().len() > 70);
        assert!(FIGURES.lines().filter(|l| !l.trim().is_empty()).all(|l| l.split('|').count() == 13));
        assert!(figures().iter().all(|f| !f.domain.is_empty() && !f.culture.is_empty()));
    }
}
