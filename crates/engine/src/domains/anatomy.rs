//! Human anatomy: bones, muscles, organs and vessels, brain regions, joints,
//! teeth, pharyngeal arches, the regions of the neural tube and the Carnegie
//! stages of embryonic development.

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

struct Part {
    names: Vec<String>,
    kind: String,
    props: Vec<(String, String)>,
}

impl Part {
    fn name(&self) -> &str {
        &self.names[0]
    }
    fn get(&self, k: &str) -> Option<&str> {
        self.props.iter().find(|p| p.0 == k).map(|p| p.1.as_str())
    }
}

// names | kind | key=value;key=value…
const DATA: &str = "\
femur;thigh bone|bone|region=thigh (lower limb);type=long bone;count=2;length=about 48 cm in adult men (about 26% of body height) — the longest and strongest bone;articulates with=hip bone (acetabulum), tibia and patella;features=head, neck, greater and lesser trochanters, linea aspera, medial and lateral condyles
tibia;shin bone;shinbone|bone|region=leg (lower limb);type=long bone;count=2;length=about 36–43 cm;articulates with=femur, fibula and talus;features=medial and lateral condyles, tibial tuberosity, medial malleolus
fibula|bone|region=leg (lower limb);type=long bone;count=2;length=about 36–40 cm;articulates with=tibia and talus;features=head, lateral malleolus; bears little weight
patella;kneecap|bone|region=knee;type=sesamoid bone (the largest);count=2;articulates with=femur;features=embedded in the quadriceps tendon
humerus|bone|region=arm (upper limb);type=long bone;count=2;length=about 33 cm;articulates with=scapula (glenoid cavity), radius and ulna;features=head, anatomical and surgical necks, greater and lesser tubercles, deltoid tuberosity, capitulum, trochlea
radius|bone|region=forearm, thumb side;type=long bone;count=2;length=about 24 cm;articulates with=humerus, ulna, scaphoid and lunate;features=head, radial tuberosity, styloid process; rotates around the ulna in pronation and supination
ulna|bone|region=forearm, little-finger side;type=long bone;count=2;length=about 26 cm;articulates with=humerus, radius and (via a disc) the carpals;features=olecranon, coronoid process, trochlear notch, styloid process
scapula;shoulder blade|bone|region=shoulder girdle;type=flat bone;count=2;articulates with=humerus and clavicle;features=glenoid cavity, acromion, coracoid process, spine, supraspinous and infraspinous fossae
clavicle;collarbone|bone|region=shoulder girdle;type=long bone;count=2;articulates with=sternum (manubrium) and scapula (acromion);features=the only long bone lying horizontally; the most commonly fractured bone
sternum;breastbone|bone|region=chest;type=flat bone;count=1;articulates with=clavicles and the costal cartilages of ribs 1–7;features=manubrium, body and xiphoid process
ribs|bone|region=chest;type=flat bones;count=24 (12 pairs: 7 true, 3 false, 2 floating);articulates with=thoracic vertebrae and (via cartilage) the sternum
vertebral column;spine;vertebrae|bone|region=back;count=33 vertebrae (7 cervical, 12 thoracic, 5 lumbar, 5 fused sacral, 4 fused coccygeal), usually counted as 26 bones in adults;features=curvatures: cervical and lumbar lordosis, thoracic and sacral kyphosis
skull;cranium|bone|region=head;count=22 bones (8 cranial, 14 facial), plus 6 ear ossicles and the hyoid nearby;features=joined by sutures; the mandible is the only movable bone
mandible;jawbone;lower jaw|bone|region=head;type=irregular bone;count=1;articulates with=temporal bones (temporomandibular joints);features=the largest and strongest facial bone; holds the lower teeth
hip bone;pelvis;coxal bone|bone|region=pelvic girdle;type=flat bone;count=2;articulates with=sacrum, the other hip bone (pubic symphysis) and femur;features=formed by the fused ilium, ischium and pubis; acetabulum
stapes|bone|region=middle ear;type=ossicle;count=2;length=about 3 mm — the smallest bone;articulates with=incus and the oval window
calcaneus;heel bone|bone|region=foot;type=tarsal (short) bone;count=2;articulates with=talus and cuboid;features=the largest tarsal; attachment of the calcaneal (Achilles) tendon
talus;ankle bone|bone|region=ankle;type=tarsal (short) bone;count=2;articulates with=tibia, fibula, calcaneus and navicular;features=no muscles attach to it
hyoid|bone|region=neck;type=irregular bone;count=1;articulates with=no other bone (suspended by muscles and ligaments)
biceps brachii;biceps|muscle|origin=short head: coracoid process of the scapula; long head: supraglenoid tubercle of the scapula;insertion=radial tuberosity and, via the bicipital aponeurosis, the deep fascia of the forearm;nerve supply=musculocutaneous nerve (C5–C6);action=flexes the elbow, supinates the forearm, weakly flexes the shoulder;blood supply=brachial artery
triceps brachii;triceps|muscle|origin=long head: infraglenoid tubercle of the scapula; lateral and medial heads: posterior surface of the humerus;insertion=olecranon of the ulna;nerve supply=radial nerve (C6–C8);action=extends the elbow;blood supply=deep brachial artery
deltoid|muscle|origin=lateral third of the clavicle, acromion and spine of the scapula;insertion=deltoid tuberosity of the humerus;nerve supply=axillary nerve (C5–C6);action=abducts the arm (beyond the first 15°); anterior fibers flex, posterior fibers extend;blood supply=posterior circumflex humeral artery
pectoralis major|muscle|origin=medial clavicle, sternum, costal cartilages 1–6, aponeurosis of the external oblique;insertion=lateral lip of the intertubercular groove of the humerus;nerve supply=lateral and medial pectoral nerves (C5–T1);action=adducts and medially rotates the arm; flexes the arm (clavicular head);blood supply=pectoral branches of the thoracoacromial artery
latissimus dorsi|muscle|origin=spinous processes of T7–L5, thoracolumbar fascia, iliac crest, lower 3–4 ribs;insertion=floor of the intertubercular groove of the humerus;nerve supply=thoracodorsal nerve (C6–C8);action=extends, adducts and medially rotates the arm;blood supply=thoracodorsal artery
trapezius|muscle|origin=occipital bone, nuchal ligament, spinous processes of C7–T12;insertion=lateral clavicle, acromion and spine of the scapula;nerve supply=accessory nerve (CN XI), with C3–C4 sensory fibers;action=elevates, retracts and rotates the scapula;blood supply=transverse cervical artery
rectus abdominis;abs|muscle|origin=pubic crest and pubic symphysis;insertion=xiphoid process and costal cartilages 5–7;nerve supply=thoracoabdominal nerves (T7–T11) and subcostal nerve (T12);action=flexes the trunk; compresses the abdomen;blood supply=superior and inferior epigastric arteries
diaphragm|muscle|origin=xiphoid process, lower six costal cartilages, L1–L3 vertebrae (crura);insertion=central tendon;nerve supply=phrenic nerve (C3–C5);action=the main muscle of inspiration: contracts and flattens to enlarge the chest cavity;blood supply=pericardiacophrenic, musculophrenic and inferior phrenic arteries
gluteus maximus|muscle|origin=posterior ilium, sacrum, coccyx and sacrotuberous ligament;insertion=iliotibial tract and gluteal tuberosity of the femur;nerve supply=inferior gluteal nerve (L5–S2);action=extends and laterally rotates the hip;blood supply=superior and inferior gluteal arteries
quadriceps femoris;quadriceps;quads|muscle|origin=rectus femoris: anterior inferior iliac spine; vastus lateralis, medialis and intermedius: shaft of the femur;insertion=tibial tuberosity via the patella and patellar ligament;nerve supply=femoral nerve (L2–L4);action=extends the knee; rectus femoris also flexes the hip;blood supply=femoral artery branches
hamstrings|muscle|origin=ischial tuberosity (biceps femoris short head: linea aspera);insertion=biceps femoris: head of the fibula; semitendinosus and semimembranosus: medial tibia;nerve supply=sciatic nerve (tibial division; common fibular division for the short head of biceps femoris);action=flex the knee and extend the hip;blood supply=perforating branches of the deep femoral artery
gastrocnemius|muscle|origin=medial and lateral condyles of the femur;insertion=posterior calcaneus via the calcaneal (Achilles) tendon;nerve supply=tibial nerve (S1–S2);action=plantarflexes the foot and flexes the knee;blood supply=sural arteries
soleus|muscle|origin=posterior head and upper shaft of the fibula, soleal line of the tibia;insertion=posterior surface of the calcaneus via the calcaneal (Achilles) tendon;nerve supply=tibial nerve (S1–S2);action=plantarflexes the foot; keeps the body upright when standing;blood supply=posterior tibial, fibular and sural arteries
tibialis anterior|muscle|origin=lateral condyle and upper lateral surface of the tibia, interosseous membrane;insertion=medial cuneiform and base of the first metatarsal;nerve supply=deep fibular nerve (L4–L5);action=dorsiflexes and inverts the foot;blood supply=anterior tibial artery
sartorius|muscle|origin=anterior superior iliac spine;insertion=upper medial surface of the tibia (pes anserinus);nerve supply=femoral nerve (L2–L3);action=flexes, abducts and laterally rotates the hip; flexes the knee — the longest muscle;blood supply=femoral artery
sternocleidomastoid|muscle|origin=manubrium of the sternum and medial clavicle;insertion=mastoid process of the temporal bone;nerve supply=accessory nerve (CN XI);action=turns the head to the opposite side; flexes the neck;blood supply=occipital and superior thyroid arteries
masseter|muscle|origin=zygomatic arch;insertion=angle and ramus of the mandible;nerve supply=masseteric nerve (mandibular division of the trigeminal nerve, V3);action=elevates the mandible (closes the jaw) — the strongest muscle by weight;blood supply=masseteric artery
supraspinatus|muscle|origin=supraspinous fossa of the scapula;insertion=greater tubercle of the humerus;nerve supply=suprascapular nerve (C5–C6);action=initiates abduction of the arm (first 15°); part of the rotator cuff;blood supply=suprascapular artery
iliopsoas|muscle|origin=psoas major: T12–L5 vertebrae; iliacus: iliac fossa;insertion=lesser trochanter of the femur;nerve supply=femoral nerve (iliacus) and anterior rami of L1–L3 (psoas);action=flexes the hip — the strongest hip flexor;blood supply=iliolumbar artery
heart|organ|system=circulatory;location=middle mediastinum of the chest, slightly left of center;mass=about 250–350 g;function=pumps blood: about 5 L per minute at rest, about 100,000 beats a day;blood supply=right and left coronary arteries;venous drainage=coronary sinus into the right atrium;nerve supply=cardiac plexus (vagus nerves and sympathetic cardiac nerves T1–T4)
lungs;lung|organ|system=respiratory;location=thoracic cavity on either side of the heart;mass=about 1.3 kg together;function=gas exchange; about 300–500 million alveoli with about 70 m² of surface;blood supply=pulmonary arteries (for gas exchange) and bronchial arteries (nourishment);venous drainage=pulmonary veins to the left atrium;nerve supply=pulmonary plexus (vagus and sympathetic trunk)
liver|organ|system=digestive;location=right upper quadrant of the abdomen, under the diaphragm;mass=about 1.5 kg — the largest internal organ;function=makes bile, stores glycogen, detoxifies, makes plasma proteins;blood supply=hepatic artery proper (25%) and hepatic portal vein (75%);venous drainage=hepatic veins to the inferior vena cava;nerve supply=hepatic plexus (vagus and celiac plexus)
stomach|organ|system=digestive;location=left upper quadrant of the abdomen;capacity=about 1–1.5 L;function=stores and churns food; secretes acid and pepsin;blood supply=left and right gastric, gastro-omental and short gastric arteries (celiac trunk);venous drainage=gastric veins to the hepatic portal vein;nerve supply=vagus nerves (parasympathetic) and celiac plexus (sympathetic, greater splanchnic T5–T9)
duodenum|organ|system=digestive;location=first part of the small intestine, C-shaped around the head of the pancreas;length=about 25 cm;function=receives bile and pancreatic juice; neutralizes stomach acid;blood supply=gastroduodenal and pancreaticoduodenal arteries;venous drainage=pancreaticoduodenal veins to the portal system
jejunum|organ|system=digestive;location=middle part of the small intestine, mostly in the left upper abdomen;length=about 2.5 m;function=absorbs most nutrients (sugars, amino acids, fatty acids);blood supply=jejunal branches of the superior mesenteric artery (long vasa recta, few arcades);venous drainage=jejunal veins to the superior mesenteric vein;nerve supply=vagus and superior mesenteric plexus;features=thick wall, many tall circular folds (plicae circulares), deep red
ileum|organ|system=digestive;location=last part of the small intestine, mostly in the right lower abdomen, ending at the ileocecal valve;length=about 3.5 m;function=absorbs vitamin B12 and bile salts; Peyer's patches;blood supply=ileal branches of the superior mesenteric artery (short vasa recta, many arcades);venous drainage=ileal veins to the superior mesenteric vein;nerve supply=vagus and superior mesenteric plexus;features=thinner wall, fewer and lower folds, more fat in the mesentery
small intestine|organ|system=digestive;location=central and lower abdomen;length=about 6–7 m (duodenum, jejunum, ileum);function=digestion and absorption of nutrients;blood supply=superior mesenteric artery (and celiac trunk for the upper duodenum);venous drainage=superior mesenteric vein, which joins the splenic vein to form the hepatic portal vein;nerve supply=vagus nerves and the superior mesenteric plexus (lesser splanchnic nerve T10–T11)
large intestine;colon|organ|system=digestive;location=frames the abdomen: cecum, ascending, transverse, descending and sigmoid colon, rectum, anal canal;length=about 1.5 m;function=absorbs water and electrolytes; stores feces;blood supply=superior mesenteric artery (to the distal transverse colon) and inferior mesenteric artery;venous drainage=superior mesenteric vein (cecum to the proximal two-thirds of the transverse colon) and inferior mesenteric vein (distal third onward), both to the hepatic portal vein; the lower rectum drains through the middle and inferior rectal veins to the internal iliac veins;nerve supply=vagus to the splenic flexure, pelvic splanchnic nerves (S2–S4) beyond; sympathetic via the superior and inferior mesenteric plexuses
gallbladder;gall bladder|organ|system=digestive;location=under the right lobe of the liver;capacity=about 30–50 mL;function=stores and concentrates bile, released into the duodenum after fatty meals;blood supply=cystic artery (usually from the right hepatic artery);venous drainage=cystic veins to the liver and portal vein;nerve supply=celiac plexus: sympathetic fibres (T7–T9 via the greater splanchnic nerve), parasympathetic fibres from the vagus nerve, and the right phrenic nerve (referred pain to the right shoulder)
pancreas|organ|system=digestive and endocrine;location=behind the stomach, across the posterior abdominal wall;length=about 15 cm;function=digestive enzymes (exocrine); insulin and glucagon from the islets of Langerhans (endocrine);blood supply=splenic artery and pancreaticoduodenal arteries;venous drainage=splenic and superior mesenteric veins
spleen|organ|system=lymphatic;location=left upper quadrant, under ribs 9–11;mass=about 150 g;function=filters blood, recycles old red cells, immune responses;blood supply=splenic artery;venous drainage=splenic vein to the hepatic portal vein
kidneys;kidney|organ|system=urinary;location=retroperitoneal, at T12–L3, the right slightly lower;mass=about 150 g each;function=filter about 180 L of plasma a day, making about 1.5 L of urine; regulate blood pressure and pH;blood supply=renal arteries;venous drainage=renal veins to the inferior vena cava
thyroid;thyroid gland|organ|system=endocrine;location=front of the neck, below the larynx;mass=about 20–25 g;function=makes thyroxine (T4), triiodothyronine (T3) and calcitonin;blood supply=superior and inferior thyroid arteries
adrenal glands;adrenal gland;adrenals|organ|system=endocrine;location=on top of each kidney;function=cortex: cortisol, aldosterone, androgens; medulla: adrenaline and noradrenaline
pituitary gland;pituitary;hypophysis|organ|system=endocrine;location=in the sella turcica of the sphenoid bone, below the hypothalamus;mass=about 0.5 g;function=the 'master gland': growth hormone, ACTH, TSH, FSH, LH, prolactin (anterior); oxytocin and ADH (posterior)
aorta|vessel|system=circulatory;function=carries oxygenated blood from the left ventricle to the whole body;location=from the left ventricle, arching over the heart and descending through the chest and abdomen;diameter=about 2.5–3 cm — the largest artery;length=about 30–40 cm;parts=ascending aorta, aortic arch, descending thoracic aorta, abdominal aorta;branches=coronary arteries, brachiocephalic trunk, left common carotid, left subclavian, intercostals, celiac trunk, superior and inferior mesenteric, renal and gonadal arteries;ends=divides into the common iliac arteries at L4
inferior vena cava|vessel|system=circulatory;location=right of the vertebral column, from L5 to the right atrium;diameter=about 2–3 cm — the largest vein;function=returns deoxygenated blood from the lower body to the right atrium;formed by=union of the common iliac veins at L5;tributaries=common iliac, lumbar, gonadal (right), renal, adrenal (right), inferior phrenic and hepatic veins
superior vena cava|vessel|system=circulatory;location=upper right mediastinum;length=about 7 cm;function=returns blood from the head, neck, arms and chest to the right atrium;formed by=union of the brachiocephalic veins
amygdala|brain region|location=deep in the medial temporal lobe, in front of the hippocampus (one in each hemisphere);system=limbic system;function=processes emotions, especially fear; forms emotional memories; social behavior;blood supply=anterior choroidal artery and branches of the middle cerebral artery;size=about 1.2 cm³ each, almond-shaped (Greek amygdale, almond)
hippocampus|brain region|location=medial temporal lobe;system=limbic system;function=forms new long-term memories; spatial navigation;size=about 3–3.5 cm³ each;blood supply=posterior cerebral artery (hippocampal arteries)
primary motor cortex;motor cortex;m1|brain region|location=precentral gyrus of the frontal lobe (Brodmann area 4);function=plans and executes voluntary movements; organized as a motor homunculus;blood supply=middle cerebral artery (lateral) and anterior cerebral artery (medial: leg area)
primary somatosensory cortex;somatosensory cortex;s1|brain region|location=postcentral gyrus of the parietal lobe (Brodmann areas 3, 1, 2);function=receives touch, pressure, pain, temperature and position sense; organized as a sensory homunculus;blood supply=middle cerebral artery (lateral) and anterior cerebral artery (medial)
primary visual cortex;visual cortex;v1|brain region|location=calcarine sulcus of the occipital lobe (Brodmann area 17);function=first cortical stage of vision;blood supply=posterior cerebral artery
primary auditory cortex;auditory cortex|brain region|location=transverse temporal (Heschl's) gyri of the temporal lobe (Brodmann areas 41, 42);function=processes sound
broca's area|brain region|location=inferior frontal gyrus of the dominant (usually left) hemisphere (Brodmann areas 44, 45);function=speech production
wernicke's area|brain region|location=posterior superior temporal gyrus of the dominant hemisphere (Brodmann area 22);function=language comprehension
ventral spinothalamic tract;anterior spinothalamic tract|neural pathway|system=somatosensory (anterolateral system);function=carries crude touch and pressure;origin=second-order neurons in the dorsal horn of the spinal cord;decussation=anterior white commissure, within one or two segments of entry;termination=ventral posterolateral (VPL) nucleus of the thalamus, then the primary somatosensory cortex;location=anterior funiculus of the spinal cord
lateral spinothalamic tract|neural pathway|system=somatosensory (anterolateral system);function=carries pain and temperature;origin=second-order neurons in the dorsal horn of the spinal cord;decussation=anterior white commissure, within one or two segments of entry;termination=ventral posterolateral (VPL) nucleus of the thalamus, then the primary somatosensory cortex;location=lateral funiculus of the spinal cord
optic radiation;geniculocalcarine tract;optic radiations|neural pathway|system=visual;origin=lateral geniculate nucleus of the thalamus;termination=primary visual cortex (V1, Brodmann area 17) on the banks of the calcarine sulcus;course=lower fibres loop forward through the temporal lobe (Meyer's loop) and carry the upper visual field; upper fibres pass through the parietal lobe;lesions=contralateral homonymous hemianopia or quadrantanopia
central taste pathway;gustatory pathway;taste pathway;central gustatory pathway|neural pathway|system=gustatory;first-order neurons=facial nerve (VII, anterior two-thirds of the tongue via the chorda tympani), glossopharyngeal nerve (IX, posterior third), vagus nerve (X, epiglottis and pharynx);relays=nucleus of the solitary tract in the medulla, then the ventral posteromedial (VPM) nucleus of the thalamus;termination=primary gustatory cortex in the anterior insula and frontal operculum
purkinje cell;cerebellum purkinje cell;cerebellar purkinje cell;purkinje cells;purkinje neuron|neuron|location=Purkinje cell layer of the cerebellar cortex;function=the sole output neuron of the cerebellar cortex; inhibits the deep cerebellar and vestibular nuclei;neurotransmitter=GABA (inhibitory);inputs=parallel fibres from granule cells and a single climbing fibre from the inferior olive;features=a large, flat, fan-shaped dendritic tree;named after=Jan Evangelista Purkyně (1837)
cone cell;retina photoreceptor cone cell;retinal cone cell;cone photoreceptor;cone cells|cell|location=retina, densest in the fovea;function=photoreceptor for colour vision and sharp daylight (photopic) vision;types=S cones (peak ≈ 420 nm), M cones (≈ 534 nm) and L cones (≈ 564 nm);count=about 6 million per human retina (rods: about 90–120 million);pigments=photopsins (opsin + 11-cis-retinal)
rod cell;retina photoreceptor rod cell;rod photoreceptor;rod cells|cell|location=retina, absent from the central fovea;function=photoreceptor for dim-light (scotopic) vision;pigment=rhodopsin (peak ≈ 498 nm);count=about 90–120 million per human retina
cerebellum|brain region|location=behind the brainstem, under the occipital lobes;function=coordination, balance, motor learning;size=about 10% of brain volume but over half of its neurons
thalamus|brain region|location=diencephalon, on both sides of the third ventricle;function=relays sensory and motor signals to the cortex; consciousness and sleep
hypothalamus|brain region|location=diencephalon, below the thalamus;function=controls the pituitary, body temperature, hunger, thirst, sleep and circadian rhythm
glenohumeral joint;shoulder joint|joint|articulating bones=head of the humerus and the glenoid cavity of the scapula;type=synovial ball-and-socket joint;movements=flexion, extension, abduction, adduction, medial and lateral rotation, circumduction — the most mobile joint;stabilized by=rotator cuff (supraspinatus, infraspinatus, teres minor, subscapularis), glenoid labrum
elbow joint;elbow|joint|articulating bones=humerus (trochlea and capitulum), ulna (trochlear notch) and radius (head);type=synovial hinge joint (with the proximal radioulnar pivot joint);movements=flexion and extension; pronation and supination at the radioulnar joint
knee joint;knee|joint|articulating bones=femur, tibia and patella;type=synovial modified hinge joint — the largest joint;movements=flexion, extension, slight rotation;stabilized by=cruciate and collateral ligaments, menisci
hip joint;hip|joint|articulating bones=head of the femur and the acetabulum of the hip bone;type=synovial ball-and-socket joint;movements=flexion, extension, abduction, adduction, rotation, circumduction
ankle joint;talocrural joint;ankle|joint|articulating bones=tibia, fibula and talus;type=synovial hinge joint;movements=dorsiflexion and plantarflexion
wrist joint;radiocarpal joint;wrist|joint|articulating bones=radius (and articular disc) with the scaphoid, lunate and triquetrum;type=synovial condyloid joint;movements=flexion, extension, abduction, adduction
temporomandibular joint;tmj;jaw joint|joint|articulating bones=condyle of the mandible and the mandibular fossa of the temporal bone;type=synovial modified hinge joint with an articular disc;movements=elevation, depression, protrusion, retraction, side-to-side
canines;canine teeth;canine;cuspids;eye teeth|teeth|count=4 in adults (2 upper, 2 lower); 4 in the primary set;eruption=primary: 16–23 months; permanent: 9–10 years (lower), 11–12 years (upper);features=single cusp and the longest root of any tooth;function=tearing and gripping food
incisors|teeth|count=8 in adults; 8 in the primary set;eruption=primary: 6–12 months; permanent: 6–9 years;function=cutting food
premolars;bicuspids|teeth|count=8 in adults; none in the primary set;eruption=10–12 years;function=crushing and tearing
molars|teeth|count=12 in adults (including 4 wisdom teeth); 8 in the primary set;eruption=permanent first molars about 6 years, second about 12 years, third (wisdom) 17–21 years;function=grinding
first pharyngeal arch;mandibular arch;1st pharyngeal arch|pharyngeal arch|nerve=trigeminal nerve (mandibular and maxillary divisions, V3 and V2);muscles=muscles of mastication (temporalis, masseter, medial and lateral pterygoids), mylohyoid, anterior belly of digastric, tensor tympani, tensor veli palatini;skeletal derivatives=Meckel's cartilage gives the malleus, incus and sphenomandibular ligament; the maxilla, zygomatic bone, squamous temporal bone and mandible form around it;artery=maxillary artery (in part)
second pharyngeal arch;hyoid arch;2nd pharyngeal arch|pharyngeal arch|nerve=facial nerve (CN VII);muscles=muscles of facial expression, stapedius, stylohyoid, posterior belly of digastric;skeletal derivatives=Reichert's cartilage gives the stapes, styloid process, stylohyoid ligament, lesser horns and upper body of the hyoid;artery=stapedial artery (temporary), hyoid artery
third pharyngeal arch;3rd pharyngeal arch|pharyngeal arch|nerve=glossopharyngeal nerve (CN IX);muscles=stylopharyngeus;skeletal derivatives=greater horns and lower body of the hyoid;artery=common carotid and proximal internal carotid arteries
fourth pharyngeal arch;4th pharyngeal arch|pharyngeal arch|nerve=vagus nerve, superior laryngeal branch;muscles=cricothyroid, pharyngeal constrictors, levator veli palatini;skeletal derivatives=thyroid cartilage;artery=aortic arch (left) and proximal right subclavian artery
sixth pharyngeal arch;6th pharyngeal arch|pharyngeal arch|nerve=vagus nerve, recurrent laryngeal branch;muscles=intrinsic muscles of the larynx (except cricothyroid);skeletal derivatives=cricoid, arytenoid and corniculate cartilages;artery=pulmonary arteries and the ductus arteriosus
intermaxillary process;intermaxillary segment|embryonic structure|formed by=fusion of the two medial nasal prominences;derived structures=philtrum of the upper lip, premaxillary part of the maxilla carrying the four upper incisors, and the primary palate
neural tube|embryonic structure|regional parts=prosencephalon (forebrain: telencephalon and diencephalon), mesencephalon (midbrain), rhombencephalon (hindbrain: metencephalon and myelencephalon) and the spinal cord;forms=days 22–28: the neural plate folds and fuses; the rostral neuropore closes about day 25, the caudal neuropore about day 27;derived structures=brain and spinal cord; neural crest cells leave its dorsal edge
";

fn parts() -> &'static Vec<Part> {
    static P: OnceLock<Vec<Part>> = OnceLock::new();
    P.get_or_init(|| {
        DATA.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| {
                let f: Vec<&str> = l.splitn(3, '|').collect();
                if f.len() != 3 {
                    return None;
                }
                Some(Part {
                    names: f[0].split(';').map(String::from).collect(),
                    kind: f[1].into(),
                    props: crate::domains::animals::key_values(f[2]),
                })
            })
            .collect()
    })
}

fn find(s: &str) -> Option<&'static Part> {
    let s = s.trim().trim_start_matches("the ").trim_start_matches("a ").trim();
    let s = s.trim_end_matches(" muscle").trim_end_matches(" bone").trim();
    parts().iter().find(|p| p.names.iter().any(|n| n == s)).or_else(|| {
        let s2 = format!("{s} bone");
        parts().iter().find(|p| p.names.iter().any(|n| *n == s2))
    })
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|x| x.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn props(p: &Part) -> Content {
    let mut r = vec![vec![text("type"), text(p.kind.clone())]];
    r.extend(p.props.iter().map(|(k, v)| vec![text(k.clone()), text(v.clone())]));
    table(None, r)
}

fn lead(p: &Part) -> String {
    let key = match p.kind.as_str() {
        "bone" => "region",
        "muscle" => "action",
        "organ" | "vessel" | "teeth" => "function",
        "brain region" => "function",
        "joint" => "articulating bones",
        "pharyngeal arch" => "nerve",
        _ => "",
    };
    let v = p.get(key).or_else(|| p.props.first().map(|x| x.1.as_str())).unwrap_or("");
    format!("{} ({}) — {}", cap(p.name()), p.kind, v)
}

fn card(p: &Part) -> Answer {
    Answer::new("anatomy", vec![pod("Input", "Input interpretation", text(format!("{} ({})", p.name(), p.kind))), primary(pod("Result", "Description", text(lead(p)))), pod("Properties", "Properties", props(p))])
}

fn prop_key(w: &str) -> &'static str {
    match w {
        "origin" | "origins" => "origin",
        "insertion" | "insertions" => "insertion",
        "nerve supply" | "innervation" | "nerve" | "nerves" => "nerve supply",
        "action" | "actions" | "function" | "functions" => "function",
        "blood supply" | "arterial supply" | "artery" | "arteries" => "blood supply",
        "venous drainage" | "veins" | "drainage" => "venous drainage",
        "articulating bones" | "articulations" | "bones" => "articulating bones",
        "location" => "location",
        "length" => "length",
        "parts" | "regional parts" | "regions" => "regional parts",
        "derived structures" | "derivatives" => "derived structures",
        _ => "",
    }
}

fn get_prop(p: &Part, key: &str) -> Option<(String, String)> {
    let alts: Vec<&str> = match key {
        "function" => vec!["function", "action"],
        "nerve supply" => vec!["nerve supply", "nerve"],
        "blood supply" => vec!["blood supply", "artery"],
        "articulating bones" => vec!["articulating bones", "articulates with"],
        "regional parts" => vec!["regional parts", "parts"],
        "derived structures" => vec!["derived structures", "skeletal derivatives"],
        "length" => vec!["length", "size"],
        k => vec![k],
    };
    alts.iter().find_map(|k| p.get(k).map(|v| (k.to_string(), v.to_string())))
}

/// Carnegie stages: (stage, days after ovulation, size mm, features)
const CARNEGIE: [(u32, &str, &str, &str); 23] = [
    (1, "1", "0.1–0.15", "fertilized oocyte (zygote)"),
    (2, "2–3", "0.1–0.2", "cleavage: 2 to 16 cells (morula)"),
    (3, "4–5", "0.1–0.2", "free blastocyst"),
    (4, "5–6", "0.1–0.2", "blastocyst attaches to the uterine lining"),
    (5, "7–12", "0.1–0.2", "implantation; bilaminar disc"),
    (6, "13–15", "0.2", "primitive streak appears; chorionic villi"),
    (7, "15–17", "0.4", "gastrulation; notochordal process"),
    (8, "17–19", "1.0–1.5", "primitive pit; neural plate and neural folds begin"),
    (9, "19–21", "1.5–2.5", "first somites (1–3); neural groove"),
    (10, "22–23", "2–3.5", "4–12 somites; neural folds begin to fuse; heart begins to beat"),
    (11, "23–26", "2.5–4.5", "13–20 somites; rostral neuropore closes; optic vesicles"),
    (12, "26–30", "3–5", "21–29 somites; caudal neuropore closes; upper limb buds appear"),
    (13, "28–32", "4–6", "30 or more somites; lower limb buds; lens placode"),
    (14, "31–35", "5–7", "lens pit and optic cup; upper limb buds elongate"),
    (15, "35–38", "7–9", "lens vesicle; hand plates; cerebral vesicles distinct"),
    (16, "37–42", "8–11", "foot plates; nasal pits face forward; retinal pigment"),
    (17, "42–44", "11–14", "finger rays; head enlarges"),
    (18, "44–48", "13–17", "toe rays; ossification begins; eyelids start"),
    (19, "48–51", "16–18", "trunk elongates and straightens"),
    (20, "51–53", "18–22", "upper limbs bend at the elbows; fingers separate"),
    (21, "53–54", "22–24", "hands and feet approach each other; fingers longer"),
    (22, "54–56", "23–28", "eyelids and external ears more developed; toes separate"),
    (23, "56–60", "27–31", "end of the embryonic period: head rounded, limbs longer, all essential organs present"),
];

fn carnegie(l: &str) -> Option<Answer> {
    let nums: Vec<u32> = re!(r"carnegie stage (\d+)").captures_iter(l).filter_map(|c| c[1].parse().ok()).collect();
    if nums.is_empty() || !l.contains("carnegie") {
        return None;
    }
    let st: Option<Vec<_>> = nums.iter().map(|n| CARNEGIE.iter().find(|c| c.0 == *n)).collect();
    let st = st?;
    let size_q = l.contains("size") || l.contains("length") || l.contains("how big");
    let main = st.iter().map(|c| if size_q { format!("stage {}: {} mm", c.0, c.2) } else { format!("stage {}: days {}, {} mm — {}", c.0, c.1, c.2, c.3) }).collect::<Vec<_>>().join(" | ");
    let t = st.iter().map(|c| vec![text(c.0.to_string()), text(format!("{} days", c.1)), text(format!("{} mm", c.2)), text(c.3)]).collect();
    Some(Answer::new("Carnegie stage", vec![
        pod("Input", "Input interpretation", text(st.iter().map(|c| format!("Carnegie stage {}", c.0)).collect::<Vec<_>>().join(" vs "))),
        primary(pod("Result", if size_q { "Embryo size (greatest length)" } else { "Description" }, text(main))),
        pod("Table", "Carnegie stages", table(Some(vec!["stage", "age (post-ovulation)", "size", "features"]), t)),
    ]))
}

fn lists(l: &str) -> Option<Answer> {
    let (title, items): (&str, &str) = match l {
        "bones" | "human bones" | "bones in the human body" | "bones of the human body" | "how many bones are in the human body" | "number of bones in the human body" => ("206 bones in the adult human body", "axial skeleton (80): skull 22, ear ossicles 6, hyoid 1, vertebral column 26, sternum 1, ribs 24 | appendicular skeleton (126): shoulder girdles 4, arms and hands 60, pelvic girdle 2, legs and feet 60"),
        "bones of the leg and the foot" | "bones of the leg and foot" | "what are the bones of the leg and the foot" | "bones of the lower limb" | "leg and foot bones" => ("bones of the leg and foot (each side: 30)", "femur | patella | tibia | fibula | 7 tarsals: talus, calcaneus, navicular, cuboid, medial, intermediate and lateral cuneiforms | 5 metatarsals | 14 phalanges"),
        "bones of the arm and hand" | "bones of the arm and the hand" | "bones of the upper limb" | "arm and hand bones" => ("bones of the arm and hand (each side: 30)", "humerus | radius | ulna | 8 carpals: scaphoid, lunate, triquetrum, pisiform, trapezium, trapezoid, capitate, hamate | 5 metacarpals | 14 phalanges"),
        "endocrine organs" | "endocrine glands" | "glands of the endocrine system" | "organs of the endocrine system" => ("endocrine organs", "hypothalamus | pituitary gland | pineal gland | thyroid gland | parathyroid glands | thymus | adrenal glands | pancreas (islets of Langerhans) | ovaries | testes"),
        "nerve supply of digestive organs" | "nerve supply of the digestive organs" | "innervation of the digestive system" | "nerve supply of the digestive system" => ("nerve supply of the digestive organs", "parasympathetic: vagus nerves to the foregut and midgut (to the splenic flexure); pelvic splanchnic nerves (S2–S4) to the hindgut | sympathetic: greater splanchnic (T5–T9) via the celiac plexus to the foregut, lesser splanchnic (T10–T11) via the superior mesenteric plexus to the midgut, lumbar splanchnics (L1–L2) via the inferior mesenteric plexus to the hindgut | intrinsic: enteric nervous system (myenteric and submucosal plexuses)"),
        "teeth" | "human teeth" | "types of teeth" => ("human teeth", "adults: 32 — 8 incisors, 4 canines, 8 premolars, 12 molars | children: 20 primary teeth — 8 incisors, 4 canines, 8 molars"),
        "pharyngeal arches" | "branchial arches" => ("pharyngeal arches", "1st (mandibular): trigeminal nerve | 2nd (hyoid): facial nerve | 3rd: glossopharyngeal nerve | 4th: vagus (superior laryngeal) | 6th: vagus (recurrent laryngeal); the 5th regresses"),
        _ => return None,
    };
    Some(Answer::new("anatomy", vec![pod("Input", "Input interpretation", text(title)), primary(pod("Result", "Result", text(items)))]))
}

/// "tooth #5", "tooth 30", "brodmann area 4"
fn numbered(l: &str) -> Option<Answer> {
    if let Some(m) = re!(r"^(?:tooth|teeth) (?:number |no\.? |#)?(\d{1,2})$|^#(\d{1,2}) tooth$").captures(l) {
        let n: u32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
        if !(1..=32).contains(&n) {
            return None;
        }
        // universal numbering: 1–8 upper right (back to front), 9–16 upper left, 17–24 lower left, 25–32 lower right
        let (quad, idx) = ((n - 1) / 8, (n - 1) % 8);
        let from_back = if quad % 2 == 0 { idx } else { 7 - idx };
        let names = ["third molar (wisdom tooth)", "second molar", "first molar", "second premolar", "first premolar", "canine", "lateral incisor", "central incisor"];
        let side = ["upper right", "upper left", "lower left", "lower right"][quad as usize];
        let fdi = (quad + 1) * 10 + (8 - from_back);
        let name = format!("{side} {}", names[from_back as usize]);
        return Some(Answer::new("tooth", vec![
            pod("Input", "Input interpretation", text(format!("tooth #{n} (Universal Numbering System)"))),
            primary(pod("Result", "Tooth", text(name.clone()))),
            pod("Properties", "Numbering systems", table(None, vec![
                vec![text("Universal (ADA)"), text(n.to_string())],
                vec![text("FDI (ISO 3950)"), text(fdi.to_string())],
                vec![text("Palmer"), text(format!("{} ({side} quadrant)", 8 - from_back))],
                vec![text("arch"), text(if quad < 2 { "maxillary (upper)" } else { "mandibular (lower)" })],
            ])),
        ]));
    }
    let m = re!(r"^(?:brodmann )?(?:area|ba) ?(\d{1,2})(?: (?:associated cortical area|cortical area|function|location))?$|^brodmann area (\d{1,2}).*$").captures(l)?;
    if !l.contains("brodmann") && !l.starts_with("ba") {
        return None;
    }
    let n: u32 = m.get(1).or(m.get(2))?.as_str().parse().ok()?;
    const AREAS: &[(&[u32], &str, &str)] = &[
        (&[1, 2, 3], "primary somatosensory cortex", "postcentral gyrus (parietal lobe); touch, pressure, pain, temperature and position sense"),
        (&[4], "primary motor cortex", "precentral gyrus (frontal lobe); voluntary movement"),
        (&[5, 7], "somatosensory association cortex (superior parietal lobule)", "integrates touch and vision for reaching and spatial attention"),
        (&[6], "premotor cortex and supplementary motor area", "planning of movement"),
        (&[8], "frontal eye fields", "voluntary eye movements"),
        (&[9, 46], "dorsolateral prefrontal cortex", "working memory and executive control"),
        (&[10], "anterior prefrontal (frontopolar) cortex", "planning, multitasking and abstract reasoning"),
        (&[11, 12], "orbitofrontal cortex", "reward value and decision making"),
        (&[17], "primary visual cortex (V1)", "calcarine sulcus (occipital lobe); first cortical stage of vision"),
        (&[18], "secondary visual cortex (V2)", "occipital lobe; visual processing"),
        (&[19], "associative visual cortex (V3, V4, V5)", "occipital lobe; motion, colour and form"),
        (&[20], "inferior temporal gyrus", "object and face recognition (ventral visual stream)"),
        (&[21], "middle temporal gyrus", "language and semantic memory"),
        (&[22], "superior temporal gyrus (posterior part: Wernicke's area)", "auditory association and language comprehension"),
        (&[23, 31], "posterior cingulate cortex", "default mode network and memory"),
        (&[24, 32, 33], "anterior cingulate cortex", "emotion, error monitoring and attention"),
        (&[25], "subgenual cingulate cortex", "mood regulation"),
        (&[28, 34], "entorhinal cortex", "gateway between the hippocampus and the neocortex; memory and navigation (grid cells)"),
        (&[37], "fusiform gyrus", "face and word recognition"),
        (&[38], "temporal pole", "semantic and social knowledge"),
        (&[39], "angular gyrus", "reading, arithmetic and language"),
        (&[40], "supramarginal gyrus", "phonological processing and body awareness"),
        (&[41, 42], "primary and secondary auditory cortex", "transverse temporal (Heschl's) gyri; hearing"),
        (&[43], "primary gustatory cortex (subcentral area)", "taste"),
        (&[44, 45], "Broca's area (pars opercularis and pars triangularis)", "inferior frontal gyrus; speech production"),
        (&[47], "pars orbitalis of the inferior frontal gyrus", "language and semantic processing"),
    ];
    let (_, name, func) = AREAS.iter().find(|a| a.0.contains(&n))?;
    Some(Answer::new("brodmann area", vec![
        pod("Input", "Input interpretation", text(format!("Brodmann area {n}"))),
        primary(pod("Result", "Cortical area", text(*name))),
        pod("Function", "Location and function", text(*func)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.strip_prefix("define ").unwrap_or(&l).to_string();
    let l = l.as_str();
    if let Some(a) = numbered(l).or_else(|| carnegie(l)).or_else(|| lists(l)) {
        return Some(a);
    }
    // "insertion of soleus", "glenohumeral joint articulating bones",
    // "venous drainage of large intestine and small intestine", "what are the regional parts of the neural tube"
    let pw = r"origins?|insertions?|nerve supply|innervation|nerves?|actions?|functions?|blood supply|arterial supply|arter(?:y|ies)|venous drainage|drainage|articulating bones|articulations|location|length|regional parts|parts|regions|derived structures|derivatives";
    static A: OnceLock<Regex> = OnceLock::new();
    static B: OnceLock<Regex> = OnceLock::new();
    let a = A.get_or_init(|| Regex::new(&format!(r"^(?:what (?:are|is) )?(?:the )?({pw}) (?:of|for) (?:the )?(.+)$")).unwrap());
    let b = B.get_or_init(|| Regex::new(&format!(r"^(?:the )?(.+?) ({pw})$")).unwrap());
    let hit = a.captures(l).map(|c| (c[1].to_string(), c[2].to_string())).or_else(|| b.captures(l).map(|c| (c[2].to_string(), c[1].to_string())));
    if let Some((w, who)) = hit {
        let key = prop_key(&w);
        let names: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\band\b|&)\s*").split(&who).filter(|s| !s.trim().is_empty()).collect();
        let ps: Option<Vec<&Part>> = names.iter().map(|n| find(n)).collect();
        if let Some(ps) = ps {
            let vals: Vec<(String, String)> = ps.iter().filter_map(|p| get_prop(p, key).map(|(_, v)| (p.name().to_string(), v))).collect();
            if !vals.is_empty() && !key.is_empty() {
                let main = if vals.len() == 1 { vals[0].1.clone() } else { vals.iter().map(|(n, v)| format!("{n}: {v}")).collect::<Vec<_>>().join(" | ") };
                let mut pods = vec![pod("Input", "Input interpretation", text(format!("{key} of {}", ps.iter().map(|p| p.name()).collect::<Vec<_>>().join(", ")))), primary(pod("Result", &cap(key), text(main)))];
                if ps.len() == 1 {
                    pods.push(pod("Properties", "Properties", props(ps[0])));
                }
                return Some(Answer::new("anatomy", pods));
            }
        }
    }
    // "femur", "ileum and jejunum", "primary motor cortex, primary somatosensory cortex"
    let names: Vec<&str> = re!(r"\s*(?:,\s*and\s+|,|\band\b|\bvs\.?\b|\bversus\b|&)\s*").split(l).filter(|s| !s.trim().is_empty()).collect();
    let ps: Option<Vec<&Part>> = names.iter().map(|n| find(n)).collect();
    let ps = ps?;
    if ps.len() == 1 {
        // ordinary words that are also anatomy ("heart", "knee") keep their other meanings only when bare and ambiguous
        // ordinary words that only sometimes mean the body part
        if matches!(l, "spine" | "ribs" | "abs" | "hip" | "colon" | "s1" | "m1" | "v1" | "canine" | "quads") {
            return None;
        }
        return Some(card(ps[0]));
    }
    let mut keys: Vec<String> = vec![];
    for p in &ps {
        for (k, _) in &p.props {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    let mut head = vec![""];
    head.extend(ps.iter().map(|p| p.name()));
    let t = keys.iter().map(|k| {
        let mut r = vec![text(k.clone())];
        r.extend(ps.iter().map(|p| text(p.get(k).unwrap_or("").to_string())));
        r
    }).collect();
    let main = ps.iter().map(|p| lead(p)).collect::<Vec<_>>().join(" | ");
    Some(Answer::new("anatomy", vec![pod("Input", "Input interpretation", text(ps.iter().map(|p| p.name()).collect::<Vec<_>>().join(" vs "))), primary(pod("Result", "Result", text(main))), pod("Table", "Comparison", table(Some(head), t))]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_parses() {
        assert_eq!(DATA.lines().filter(|l| !l.trim().is_empty()).count(), parts().len());
        assert!(parts().iter().all(|p| !p.props.is_empty()));
    }
}
