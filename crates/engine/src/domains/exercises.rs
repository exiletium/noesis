//! Exercises: yoga poses, Pilates mat exercises and strength-training moves,
//! with the muscles they work, how to do them and what they are for.
//!
//! "Pincha Mayurasana", "how do you do a squat", "double leg stretch",
//! "hundred pilates primary contracted muscles", "Sun Salutation A",
//! "yoga poses to strengthen abdominal muscles", "advanced Pilates exercises",
//! "isolated weight training exercises", "exercises for the glutes".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// names (first is the display name; `~` = not taken alone) | kind |
/// category | level | muscles worked | how to do it | benefits
type Row = (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str);

const EXERCISES: &[Row] = &[
    // yoga
    ("Mountain Pose;Tadasana;mountain", "yoga", "standing pose", "beginner", "quadriceps;glutes;core;calves", "stand with feet together, weight even, thighs firm, spine long, arms by the sides", "posture; balance; body awareness"),
    ("Downward-Facing Dog;Adho Mukha Svanasana;downward dog;down dog", "yoga", "inversion", "beginner", "shoulders;hamstrings;calves;back;arms", "from hands and knees lift the hips up and back into an inverted V, hands shoulder-width, heels reaching to the floor", "stretches the hamstrings, calves and spine; strengthens the arms and shoulders"),
    ("Upward-Facing Dog;Urdhva Mukha Svanasana;upward dog;up dog", "yoga", "backbend", "beginner", "back;triceps;glutes;chest", "lying face down, press the hands into the floor and straighten the arms, lifting the chest and thighs off the floor", "opens the chest; strengthens the spine and arms; posture"),
    ("Cobra Pose;Bhujangasana;cobra", "yoga", "backbend", "beginner", "back;glutes;chest;abdominal", "lying face down with hands under the shoulders, lift the chest using the back muscles, elbows bent and close to the body", "strengthens the spine; opens the chest; posture"),
    ("Child's Pose;Balasana;childs pose", "yoga", "resting pose", "beginner", "back;hips;thighs", "kneel, sit back on the heels and fold forward, forehead to the floor, arms forward or alongside the body", "rest; gently stretches the back, hips and thighs"),
    ("Warrior I;Virabhadrasana I;warrior 1;warrior one", "yoga", "standing pose", "beginner", "quadriceps;glutes;shoulders;core;hip flexors", "lunge with the back foot turned out, hips facing forward, front knee over the ankle, arms raised overhead", "strengthens the legs; stretches the hip flexors; balance"),
    ("Warrior II;Virabhadrasana II;warrior 2;warrior two", "yoga", "standing pose", "beginner", "quadriceps;glutes;shoulders;adductors", "wide stance, front foot forward and back foot parallel to the back edge, front knee bent over the ankle, arms extended to the sides", "strengthens the legs and shoulders; stamina; opens the hips"),
    ("Warrior III;Virabhadrasana III;warrior 3;warrior three", "yoga", "balancing pose", "intermediate", "hamstrings;glutes;core;back;ankles", "balance on one leg with the torso and the other leg parallel to the floor, arms forward", "balance; strengthens the legs, core and back; posture"),
    ("Triangle Pose;Trikonasana;Utthita Trikonasana;triangle", "yoga", "standing pose", "beginner", "hamstrings;obliques;adductors;quadriceps", "wide stance with straight legs, hinge sideways over the front leg, lower hand to the shin or floor, top arm reaching up", "stretches the hamstrings, hips and spine; strengthens the legs"),
    ("Extended Side Angle Pose;Utthita Parsvakonasana;side angle", "yoga", "standing pose", "beginner", "quadriceps;glutes;obliques;adductors", "from Warrior II lower the forearm to the front thigh or the hand to the floor, top arm reaching over the ear", "strengthens the legs; stretches the side body"),
    ("Tree Pose;Vrksasana;tree", "yoga", "balancing pose", "beginner", "calves;ankles;core;adductors;glutes", "stand on one leg with the other sole against the inner thigh or calf (not the knee), hands at the chest or overhead", "balance; strengthens the ankles and legs; focus"),
    ("Chair Pose;Utkatasana;chair", "yoga", "standing pose", "beginner", "quadriceps;glutes;back;shoulders;calves", "bend the knees as if sitting in a chair, weight in the heels, arms raised overhead", "strengthens the legs and back"),
    ("Plank Pose;Phalakasana;plank", "yoga", "arm balance", "beginner", "abdominal;core;shoulders;arms;back", "hold the top of a push-up with the body in a straight line from head to heels", "strengthens the core, arms and shoulders"),
    ("Four-Limbed Staff Pose;Chaturanga Dandasana;chaturanga", "yoga", "arm balance", "intermediate", "triceps;chest;shoulders;core", "from plank lower until the elbows are bent at 90° and close to the ribs, body straight", "strengthens the arms, wrists and core"),
    ("Side Plank;Vasisthasana", "yoga", "arm balance", "intermediate", "obliques;shoulders;core;abdominal", "balance on one hand and the outer edge of one foot, body in a straight line, top arm raised", "strengthens the obliques, arms and wrists; balance"),
    ("Bridge Pose;Setu Bandha Sarvangasana;bridge", "yoga", "backbend", "beginner", "glutes;hamstrings;back", "lying on the back with knees bent and feet flat, lift the hips, shoulders and feet staying down", "strengthens the glutes and back; opens the chest and hip flexors"),
    ("Wheel Pose;Urdhva Dhanurasana;upward bow;wheel", "yoga", "backbend", "advanced", "back;glutes;shoulders;arms;quadriceps", "lying on the back, hands by the ears, press up into a full arch", "strengthens the arms, legs and spine; opens the chest and shoulders"),
    ("Camel Pose;Ustrasana;camel", "yoga", "backbend", "intermediate", "back;quadriceps;hip flexors;chest", "kneel with hips over the knees, arch back and reach the hands to the heels", "stretches the front of the body; strengthens the back"),
    ("Bow Pose;Dhanurasana;bow", "yoga", "backbend", "intermediate", "back;glutes;hamstrings;chest", "lying face down, hold the ankles and lift the chest and thighs, pulling into a bow", "strengthens the back; stretches the front of the body"),
    ("Locust Pose;Salabhasana;locust", "yoga", "backbend", "beginner", "back;glutes;hamstrings", "lying face down, lift the head, chest, arms and legs off the floor", "strengthens the back muscles; posture"),
    ("Boat Pose;Navasana;Paripurna Navasana;boat", "yoga", "seated pose", "intermediate", "abdominal;hip flexors;core;back", "balance on the sit bones with the legs lifted and the torso leaning back, forming a V, arms forward", "strengthens the abdominal muscles and hip flexors"),
    ("Crow Pose;Bakasana;Kakasana;crane pose;crow", "yoga", "arm balance", "intermediate", "arms;wrists;core;abdominal", "squat, place the hands down, rest the knees on the backs of the upper arms and lift the feet", "strengthens the arms, wrists and core; balance"),
    ("Headstand;Sirsasana;Salamba Sirsasana", "yoga", "inversion", "advanced", "shoulders;core;neck;arms", "forearms on the floor with fingers interlaced, crown of the head down, lift the legs to vertical", "strengthens the shoulders and core; balance"),
    ("Shoulder Stand;Salamba Sarvangasana;Sarvangasana", "yoga", "inversion", "intermediate", "shoulders;core;back", "lying on the back, lift the legs and hips overhead and support the back with the hands, body vertical", "strengthens the shoulders and core"),
    ("Plow Pose;Halasana;plough pose", "yoga", "inversion", "intermediate", "back;hamstrings;shoulders", "from shoulder stand lower the feet to the floor behind the head", "stretches the spine and hamstrings"),
    ("Fish Pose;Matsyasana", "yoga", "backbend", "beginner", "back;neck;chest", "lying on the back, lift the chest on the forearms and lower the crown of the head to the floor", "opens the chest and throat; counterpose to shoulder stand"),
    ("Corpse Pose;Savasana;Shavasana;corpse", "yoga", "resting pose", "beginner", "", "lie flat on the back, arms by the sides, palms up, and relax completely", "relaxation; usually the final pose of a practice"),
    ("Lotus Pose;Padmasana;lotus", "yoga", "seated pose", "advanced", "hips;knees;ankles", "sit cross-legged with each foot on the opposite thigh", "meditation posture; opens the hips"),
    ("Easy Pose;Sukhasana", "yoga", "seated pose", "beginner", "hips;back", "sit cross-legged with the spine upright", "meditation posture"),
    ("Pigeon Pose;Eka Pada Rajakapotasana;pigeon", "yoga", "hip opener", "intermediate", "hips;glutes;hip flexors;piriformis", "one shin forward across the mat, the other leg extended straight back, hips square", "opens the hips; stretches the glutes and hip flexors"),
    ("Seated Forward Bend;Paschimottanasana", "yoga", "seated pose", "beginner", "hamstrings;back;calves", "sit with legs straight forward and fold over them from the hips", "stretches the hamstrings and spine; calming"),
    ("Standing Forward Bend;Uttanasana;forward fold", "yoga", "standing pose", "beginner", "hamstrings;calves;back", "from standing fold forward from the hips, head hanging", "stretches the hamstrings and back"),
    ("Cat Pose;Marjaryasana;cat-cow;cat cow", "yoga", "warm-up", "beginner", "back;abdominal;neck", "on hands and knees, round the spine up (cat), then arch it with the belly down (cow), with the breath", "mobilises the spine; warm-up"),
    ("Half Moon Pose;Ardha Chandrasana;half moon", "yoga", "balancing pose", "intermediate", "glutes;obliques;ankles;hamstrings;core", "balance on one leg and one hand, the other leg lifted parallel to the floor and the body open to the side", "balance; strengthens the legs and core"),
    ("Eagle Pose;Garudasana;eagle", "yoga", "balancing pose", "intermediate", "quadriceps;glutes;shoulders;ankles", "sit back as in chair pose, wrap one leg around the other and one arm around the other", "balance; stretches the shoulders and upper back"),
    ("Dancer Pose;Natarajasana;Lord of the Dance;dancer", "yoga", "balancing pose", "intermediate", "quadriceps;hip flexors;glutes;shoulders;back", "balance on one leg, hold the other foot behind you and kick it up as the torso leans forward", "balance; opens the chest, shoulders and hip flexors"),
    ("Forearm Stand;Pincha Mayurasana;peacock feather pose", "yoga", "inversion", "advanced", "shoulders;core;arms;back", "balance on the forearms with the legs vertical overhead", "strengthens the shoulders, arms and core; balance"),
    ("Peacock Pose;Mayurasana", "yoga", "arm balance", "advanced", "arms;wrists;core;abdominal;back", "balance the body horizontally on the hands with the elbows pressed into the abdomen", "strengthens the wrists, forearms and core"),
    ("Happy Baby Pose;Ananda Balasana;happy baby", "yoga", "hip opener", "beginner", "hips;adductors;back", "lying on the back, hold the outer feet with the knees bent toward the armpits", "opens the hips; releases the lower back"),
    ("Bound Angle Pose;Baddha Konasana;cobbler's pose;butterfly pose", "yoga", "hip opener", "beginner", "adductors;hips", "sit with the soles of the feet together and the knees falling out to the sides", "opens the hips and groin"),
    ("Garland Pose;Malasana;yogi squat", "yoga", "hip opener", "beginner", "hips;adductors;ankles;back", "deep squat with feet close and elbows pressing the knees apart, palms together", "opens the hips; stretches the ankles and lower back"),
    ("Half Lord of the Fishes Pose;Ardha Matsyendrasana;seated twist", "yoga", "twist", "beginner", "obliques;back;hips", "sit and cross one foot over the opposite knee, twisting toward the bent knee", "mobilises the spine"),
    ("Hero Pose;Virasana", "yoga", "seated pose", "beginner", "quadriceps;ankles;knees", "kneel and sit between the heels, spine upright", "stretches the thighs and ankles"),
    ("Wide-Legged Forward Bend;Prasarita Padottanasana", "yoga", "standing pose", "beginner", "hamstrings;adductors;back", "with the feet wide apart, fold forward and bring the hands or head toward the floor", "stretches the hamstrings and inner thighs"),
    ("Pyramid Pose;Parsvottanasana;intense side stretch", "yoga", "standing pose", "beginner", "hamstrings;hips;back", "front foot forward, back foot angled, both legs straight; fold over the front leg", "stretches the hamstrings"),
    ("Revolved Triangle;Parivrtta Trikonasana", "yoga", "twist", "intermediate", "hamstrings;obliques;back", "from a short triangle stance twist and bring the opposite hand to the front foot", "stretches the hamstrings; spinal twist; balance"),
    ("Firefly Pose;Tittibhasana", "yoga", "arm balance", "advanced", "arms;core;hip flexors;hamstrings", "balance on the hands with the legs straight out to the sides over the arms", "strengthens the arms and core; stretches the hamstrings"),
    ("Staff Pose;Dandasana", "yoga", "seated pose", "beginner", "back;core;quadriceps", "sit with the legs straight forward and the spine upright, hands beside the hips", "posture; foundation of the seated poses"),
    ("Upward Plank Pose;Purvottanasana;reverse plank", "yoga", "arm balance", "intermediate", "glutes;hamstrings;shoulders;triceps;back", "from staff pose lift the hips so the body forms a straight upward-facing line", "strengthens the arms and back of the body; opens the shoulders"),
    // sequences
    ("Sun Salutation A;Surya Namaskar A;sun salutation;Surya Namaskar", "yoga", "sequence", "beginner", "shoulders;arms;core;hamstrings;back", "Mountain Pose (Tadasana) → Upward Salute (Urdhva Hastasana) → Standing Forward Bend (Uttanasana) → Half Forward Bend (Ardha Uttanasana) → Four-Limbed Staff (Chaturanga) → Upward-Facing Dog → Downward-Facing Dog (5 breaths) → Half Forward Bend → Standing Forward Bend → Upward Salute → Mountain Pose", "warm-up; links movement and breath"),
    ("Sun Salutation B;Surya Namaskar B", "yoga", "sequence", "intermediate", "quadriceps;shoulders;arms;core", "Chair Pose (Utkatasana) → Forward Bend → Half Forward Bend → Chaturanga → Upward Dog → Downward Dog → Warrior I right → Chaturanga → Upward Dog → Downward Dog → Warrior I left → Chaturanga → Upward Dog → Downward Dog (5 breaths) → Half Forward Bend → Forward Bend → Chair Pose → Mountain Pose", "builds heat and strength"),
    ("Ashtanga Primary Series;primary series;Yoga Chikitsa;ashtanga", "yoga", "sequence", "intermediate", "", "opening: Sun Salutations A and B; standing: Padangusthasana, Padahastasana, Utthita Trikonasana, Parivrtta Trikonasana, Utthita Parsvakonasana, Parivrtta Parsvakonasana, Prasarita Padottanasana A–D, Parsvottanasana, Utthita Hasta Padangusthasana, Ardha Baddha Padmottanasana, Utkatasana, Virabhadrasana I and II; seated: Dandasana, Paschimottanasana, Purvottanasana, Ardha Baddha Padma Paschimottanasana, Tiriang Mukha Eka Pada Paschimottanasana, Janu Sirsasana A–C, Marichyasana A–D, Navasana, Bhujapidasana, Kurmasana, Supta Kurmasana, Garbha Pindasana, Kukkutasana, Baddha Konasana, Upavistha Konasana, Supta Konasana, Supta Padangusthasana, Ubhaya Padangusthasana, Urdhva Mukha Paschimottanasana, Setu Bandhasana; finishing: Urdhva Dhanurasana, Paschimottanasana, Salamba Sarvangasana, Halasana, Karnapidasana, Urdhva Padmasana, Pindasana, Matsyasana, Uttana Padasana, Sirsasana, Baddha Padmasana, Padmasana, Utplutih, Savasana", "\"yoga therapy\": detoxifies and aligns the body (Pattabhi Jois)"),
    // Pilates (the classical mat order of Joseph Pilates' Return to Life)
    ("The Hundred;hundred;hundreds", "Pilates", "mat", "beginner", "abdominal;transversus abdominis;hip flexors;shoulders", "lying on the back, head and shoulders curled up, legs at tabletop or extended at 45°, pump the straight arms up and down: breathe in for 5 pumps and out for 5, 10 times (100 pumps)", "warms up the body; abdominal endurance; breathing control"),
    ("Roll Up;roll-up;pilates roll up", "Pilates", "mat", "beginner", "abdominal;hip flexors;back", "lying with arms overhead, curl up one vertebra at a time to a forward reach over the legs, then roll back down", "abdominal strength; spinal articulation; hamstring stretch"),
    ("Roll Over;rollover", "Pilates", "mat", "intermediate", "abdominal;back;hamstrings", "lying on the back, lift the straight legs over the head toward the floor behind, open the legs and roll down slowly", "abdominal control; spinal flexibility"),
    ("Single Leg Circles;one leg circle;leg circles", "Pilates", "mat", "beginner", "hip flexors;abdominal;adductors;quadriceps", "lying on the back with one leg to the ceiling, draw circles with it while the pelvis stays still", "hip mobility; pelvic stability"),
    ("Rolling Like a Ball;rolling like a ball", "Pilates", "mat", "beginner", "abdominal;back", "balance in a tight ball holding the shins, roll back to the shoulder blades and return to balance", "massages the spine; abdominal control; balance"),
    ("Single Leg Stretch;single-leg stretch", "Pilates", "mat", "beginner", "abdominal;obliques;hip flexors", "head and shoulders up, pull one knee to the chest while the other leg extends at 45°, then switch", "abdominal strength; coordination"),
    ("Double Leg Stretch;double-leg stretch", "Pilates", "mat", "beginner", "abdominal;transversus abdominis;hip flexors;shoulders", "curled up with knees to the chest, extend the arms overhead and the legs forward at the same time, then circle the arms around and hug the knees back in", "abdominal strength and endurance; coordination"),
    ("Single Straight Leg Stretch;scissors stretch;single straight leg", "Pilates", "mat", "intermediate", "abdominal;hamstrings;hip flexors", "head and shoulders up, both legs straight, pull one leg toward you while the other lowers, switching in a scissor motion", "abdominal strength; hamstring flexibility"),
    ("Double Straight Leg Stretch;lower lift;double straight leg", "Pilates", "mat", "intermediate", "abdominal;transversus abdominis;hip flexors", "hands behind the head, straight legs to the ceiling; lower them toward the floor and lift them back, keeping the lower back down", "lower abdominal strength"),
    ("Criss-Cross;criss cross;crisscross", "Pilates", "mat", "intermediate", "obliques;abdominal", "hands behind the head, twist the elbow toward the opposite bent knee while the other leg extends, then switch", "oblique strength; waist"),
    ("Spine Stretch Forward;spine stretch", "Pilates", "mat", "beginner", "back;abdominal;hamstrings", "sit tall with legs apart, round forward one vertebra at a time, reaching the arms, then restack the spine", "spinal flexibility; posture"),
    ("Open Leg Rocker;open-leg rocker", "Pilates", "mat", "intermediate", "abdominal;back;hamstrings", "balance holding the ankles with straight legs in a V, roll back to the shoulders and return to balance", "balance; abdominal control"),
    ("Corkscrew", "Pilates", "mat", "intermediate", "abdominal;obliques", "lying on the back with the legs together toward the ceiling, circle them to one side, down and around to the other", "abdominal control; mobility of the spine"),
    ("Saw;the saw", "Pilates", "mat", "beginner", "obliques;back;hamstrings", "sit with legs wide and arms out to the sides, twist and reach one hand past the opposite little toe, sawing", "spinal rotation; hamstring stretch"),
    ("Swan Dive;swan", "Pilates", "mat", "advanced", "back;glutes;hamstrings", "lying face down, lift the chest into an extension and rock forward and back with the arms out", "back strength; spinal extension"),
    ("Single Leg Kick", "Pilates", "mat", "beginner", "hamstrings;glutes;back", "lying face down propped on the forearms, kick each heel toward the buttock twice", "hamstring strength; stretches the thighs"),
    ("Double Leg Kick", "Pilates", "mat", "intermediate", "back;hamstrings;glutes;shoulders", "face down with hands clasped behind the back, kick both heels three times, then extend the arms back and lift the chest", "back strength; opens the chest and shoulders"),
    ("Neck Pull", "Pilates", "mat", "intermediate", "abdominal;back;hamstrings", "a roll-up with the hands behind the head and the legs straight", "abdominal strength; spinal articulation"),
    ("Scissors;pilates scissors", "Pilates", "mat", "advanced", "abdominal;hip flexors;hamstrings;back", "with the hips supported high overhead, split the straight legs forward and back", "hip flexor stretch; abdominal control"),
    ("Bicycle;pilates bicycle;bicycle pilates", "Pilates", "mat", "advanced", "abdominal;hip flexors;glutes;hamstrings", "with the hips supported high, pedal the legs in a large backward cycling motion", "hip flexibility; abdominal and back control"),
    ("Shoulder Bridge", "Pilates", "mat", "intermediate", "glutes;hamstrings;back;abdominal", "from a bridge, kick one straight leg to the ceiling and lower it, keeping the pelvis level", "glute and hamstring strength; pelvic stability"),
    ("Spine Twist;spine twist", "Pilates", "mat", "beginner", "obliques;back", "sit tall with the legs straight forward and arms out to the sides, twist the torso in two pulses each way", "spinal rotation; posture"),
    ("Jackknife", "Pilates", "mat", "advanced", "abdominal;back;glutes", "from lying, lift the legs over the head and then shoot them up to the ceiling, rolling down slowly", "abdominal control; spinal articulation"),
    ("Side Kick;side kick series;side kicks", "Pilates", "mat", "beginner", "glutes;abductors;hip flexors;obliques", "lying on the side, kick the top leg forward twice and sweep it back, keeping the torso still", "hip strength and stability"),
    ("Teaser", "Pilates", "mat", "advanced", "abdominal;hip flexors;back", "roll up to balance on the sit bones in a V with legs and arms reaching, then roll down with control", "abdominal strength; balance; control"),
    ("Hip Circles;hip twist", "Pilates", "mat", "advanced", "abdominal;obliques;shoulders", "lean back on the hands with the legs raised together and circle them", "abdominal control; shoulder stability"),
    ("Swimming;pilates swimming", "Pilates", "mat", "beginner", "back;glutes;shoulders;hamstrings", "face down with arms and legs extended and lifted, flutter opposite arm and leg quickly", "back strength; posture"),
    ("Leg Pull Front;leg pull down", "Pilates", "mat", "intermediate", "abdominal;shoulders;glutes;hamstrings", "in a front plank, lift one straight leg at a time", "core and shoulder strength"),
    ("Leg Pull Back;leg pull up", "Pilates", "mat", "advanced", "glutes;hamstrings;shoulders;triceps", "in a reverse plank, kick one straight leg up at a time", "back-of-body strength; shoulder stability"),
    ("Kneeling Side Kick", "Pilates", "mat", "advanced", "obliques;glutes;abductors;shoulders", "kneel, lean sideways onto one hand and kick the top leg forward and back", "balance; hip and side-body strength"),
    ("Side Bend;mermaid", "Pilates", "mat", "intermediate", "obliques;quadratus lumborum;shoulders;latissimus dorsi;abdominal", "sit on one hip, press up into a side plank with the top arm arcing overhead, then lower with control", "strengthens the obliques and shoulders; stretches the side body"),
    ("Boomerang", "Pilates", "mat", "advanced", "abdominal;back;hamstrings", "seated with crossed legs, roll over, switch the leg cross, roll up to a teaser and reach the arms back", "abdominal control; coordination"),
    ("Seal;the seal", "Pilates", "mat", "beginner", "abdominal;back", "balance holding the ankles with knees apart, clap the feet three times, roll back and clap again", "massages the spine; abdominal control"),
    ("Crab", "Pilates", "mat", "advanced", "abdominal;back", "a rolling exercise with crossed legs that ends on the knees with the head touching the mat", "abdominal control; spinal massage"),
    ("Rocking", "Pilates", "mat", "advanced", "back;glutes;quadriceps", "face down holding the ankles in a bow, rock forward and back", "back strength; stretches the front of the body"),
    ("Control Balance", "Pilates", "mat", "advanced", "abdominal;back;hamstrings", "from a roll-over position hold one foot and lift the other leg to the ceiling, switching", "balance; abdominal and back control"),
    ("Pilates Push-Up", "Pilates", "mat", "intermediate", "chest;triceps;shoulders;abdominal", "roll down to the floor, walk the hands out to a plank, do push-ups, and walk back", "upper-body and core strength"),
    // strength training
    ("Squat;squats;bodyweight squat;back squat", "strength", "compound", "beginner", "quadriceps;glutes;hamstrings;adductors;core", "feet shoulder-width apart, chest up, push the hips back and bend the knees until the thighs are about parallel to the floor, knees tracking over the toes, then drive up through the heels", "leg and glute strength; functional movement"),
    ("Deadlift;deadlifts", "strength", "compound", "intermediate", "hamstrings;glutes;back;trapezius;forearms", "with the bar over the mid-foot, hinge at the hips with a flat back, grip the bar and stand up by driving the hips forward", "posterior-chain and grip strength"),
    ("Bench Press;bench", "strength", "compound", "beginner", "chest;triceps;shoulders", "lying on a bench, lower the bar to the mid-chest and press it back up to straight arms", "upper-body pushing strength"),
    ("Overhead Press;military press;shoulder press", "strength", "compound", "beginner", "shoulders;triceps;trapezius;core", "press a barbell or dumbbells from the shoulders to straight arms overhead", "shoulder strength"),
    ("Push-Up;push-ups;pushup;pushups;press-up", "strength", "compound", "beginner", "chest;triceps;shoulders;core", "from a plank with hands slightly wider than the shoulders, lower the chest to the floor and push back up, body straight", "upper-body strength; core stability"),
    ("Pull-Up;pull-ups;pullup;pullups", "strength", "compound", "intermediate", "latissimus dorsi;biceps;back;forearms", "hang from a bar with an overhand grip and pull until the chin is over the bar", "back and arm strength"),
    ("Chin-Up;chin-ups;chinup", "strength", "compound", "intermediate", "biceps;latissimus dorsi;back", "like a pull-up with an underhand (palms toward you) grip", "biceps and back strength"),
    ("Lunge;lunges;forward lunge", "strength", "compound", "beginner", "quadriceps;glutes;hamstrings;adductors", "step forward and lower until both knees are at about 90°, then push back to standing", "leg strength; balance"),
    ("Bent-Over Row;barbell row;row;dumbbell row", "strength", "compound", "beginner", "latissimus dorsi;back;biceps;trapezius", "hinged forward with a flat back, pull the weight to the lower ribs", "back strength; posture"),
    ("Dip;dips;parallel bar dip", "strength", "compound", "intermediate", "triceps;chest;shoulders", "support yourself on parallel bars and lower until the elbows are at 90°, then press back up", "triceps and chest strength"),
    ("Hip Thrust;hip thrusts;glute bridge", "strength", "compound", "beginner", "glutes;hamstrings", "upper back on a bench and a weight across the hips, drive the hips up until the body is level", "glute strength"),
    ("Leg Press", "strength", "compound", "beginner", "quadriceps;glutes;hamstrings", "on the machine, lower the platform until the knees are at about 90° and press it back", "leg strength"),
    ("Burpee;burpees", "strength", "compound", "intermediate", "quadriceps;chest;shoulders;core;glutes", "squat, kick the feet back to a plank, do a push-up, jump the feet in and jump up", "conditioning; full-body"),
    ("Kettlebell Swing;kettlebell swings", "strength", "compound", "intermediate", "glutes;hamstrings;back;core;shoulders", "hinge and swing the kettlebell between the legs, then snap the hips forward to swing it to chest height", "hip power; conditioning"),
    ("Back Extension;back extensions;hyperextension;hyperextensions", "strength", "isolation", "beginner", "back;erector spinae;glutes;hamstrings", "on a back-extension bench with the hips on the pad, lower the torso by bending at the hips and raise it until the body is straight (not past it)", "lower-back strength; posture"),
    ("Biceps Curl;bicep curl;curls;dumbbell curl", "strength", "isolation", "beginner", "biceps;forearms", "with the elbows at the sides, bend the arms to lift the weights to the shoulders", "arm strength"),
    ("Triceps Extension;tricep extension;skull crusher;overhead triceps extension", "strength", "isolation", "beginner", "triceps", "with the upper arm still, straighten the elbow against resistance", "arm strength"),
    ("Lateral Raise;lateral raises;side raise", "strength", "isolation", "beginner", "shoulders;deltoids", "raise dumbbells out to the sides to shoulder height with slightly bent elbows", "shoulder width and strength"),
    ("Leg Curl;hamstring curl", "strength", "isolation", "beginner", "hamstrings", "on the machine, bend the knees against the pad", "hamstring strength"),
    ("Leg Extension", "strength", "isolation", "beginner", "quadriceps", "on the machine, straighten the knees against the pad", "quadriceps strength"),
    ("Calf Raise;calf raises", "strength", "isolation", "beginner", "calves", "rise onto the balls of the feet and lower slowly", "calf strength; ankle stability"),
    ("Chest Fly;dumbbell fly;pec fly;pec deck", "strength", "isolation", "beginner", "chest;shoulders", "with slightly bent elbows, open the arms wide and bring them together in an arc", "chest strength and stretch"),
    ("Lat Pulldown;lat pull-down", "strength", "compound", "beginner", "latissimus dorsi;biceps;back", "seated at the machine, pull the bar to the upper chest", "back strength"),
    ("Crunch;crunches", "strength", "isolation", "beginner", "abdominal", "lying with knees bent, curl the head and shoulders off the floor", "abdominal strength"),
    ("Sit-Up;sit-ups;situp", "strength", "isolation", "beginner", "abdominal;hip flexors", "lying with knees bent, curl all the way up to sitting", "abdominal and hip-flexor strength"),
    ("Russian Twist;russian twists", "strength", "isolation", "beginner", "obliques;abdominal", "seated leaning back with the feet up, rotate the torso side to side", "oblique strength"),
    ("Mountain Climber;mountain climbers", "strength", "compound", "beginner", "core;abdominal;shoulders;hip flexors", "in a plank, drive the knees toward the chest alternately and quickly", "conditioning; core"),
    ("Shrug;shrugs", "strength", "isolation", "beginner", "trapezius", "holding weights, lift the shoulders straight up toward the ears", "upper-trapezius strength"),
    ("Wall Sit;wall sits", "strength", "isolation", "beginner", "quadriceps;glutes", "back against a wall, slide down until the thighs are parallel to the floor and hold", "isometric leg endurance"),
];

struct Ex {
    names: Vec<&'static str>,
    bare: bool,
    kind: &'static str,
    category: &'static str,
    level: &'static str,
    muscles: Vec<&'static str>,
    how: &'static str,
    benefits: &'static str,
}

fn all() -> &'static Vec<Ex> {
    static A: OnceLock<Vec<Ex>> = OnceLock::new();
    A.get_or_init(|| {
        EXERCISES
            .iter()
            .map(|r| Ex {
                bare: !r.0.starts_with('~'),
                names: r.0.trim_start_matches('~').split(';').collect(),
                kind: r.1,
                category: r.2,
                level: r.3,
                muscles: r.4.split(';').filter(|s| !s.is_empty()).collect(),
                how: r.5,
                benefits: r.6,
            })
            .collect()
    })
}

fn norm(s: &str) -> String {
    s.to_lowercase().replace(['-', '\''], " ").replace("’", " ").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find(s: &str) -> Option<&'static Ex> {
    let s = norm(s.trim().trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an "));
    let s = s.trim_end_matches(" exercise").trim_end_matches(" yoga pose").trim_end_matches(" pose").trim_end_matches(" in yoga").trim_end_matches(" in pilates").trim_end_matches(" pilates").trim_end_matches(" yoga").to_string();
    let s = s.trim_start_matches("pilates ").trim_start_matches("yoga ").to_string();
    for e in all() {
        for n in &e.names {
            let n = norm(n);
            if n == s || n.trim_end_matches(" pose") == s {
                return Some(e);
            }
        }
    }
    None
}

/// Muscle groups as people name them, with the names used in the table.
fn muscle(s: &str) -> Option<&'static [&'static str]> {
    let s = s.trim().trim_end_matches(" muscles").trim_end_matches(" muscle").trim();
    Some(match s {
        "abs" | "abdominals" | "abdominal" | "abdomen" | "stomach" | "belly" | "core" | "abdominal muscles" | "six pack" => &["abdominal", "core", "transversus abdominis", "obliques"],
        "obliques" | "oblique" | "waist" | "sides" => &["obliques"],
        "glutes" | "glute" | "buttocks" | "butt" | "bum" | "gluteus maximus" => &["glutes"],
        "legs" | "leg" | "thighs" => &["quadriceps", "hamstrings", "glutes", "calves", "adductors"],
        "quads" | "quadriceps" | "front of the thighs" => &["quadriceps"],
        "hamstrings" | "hamstring" | "back of the thighs" => &["hamstrings"],
        "calves" | "calf" => &["calves"],
        "back" | "lower back" | "spine" | "upper back" => &["back", "erector spinae", "latissimus dorsi", "trapezius"],
        "lats" | "latissimus dorsi" => &["latissimus dorsi"],
        "chest" | "pecs" | "pectorals" => &["chest"],
        "shoulders" | "shoulder" | "deltoids" | "delts" => &["shoulders", "deltoids"],
        "arms" | "arm" => &["arms", "biceps", "triceps", "forearms"],
        "biceps" | "bicep" => &["biceps"],
        "triceps" | "tricep" => &["triceps"],
        "hips" | "hip" | "hip flexors" => &["hips", "hip flexors"],
        "inner thighs" | "adductors" => &["adductors"],
        "trapezius" | "traps" | "neck" => &["trapezius", "neck"],
        _ => return None,
    })
}

fn kind_of(s: &str) -> Option<&'static str> {
    let s = s.trim();
    if s.contains("pilates") {
        Some("Pilates")
    } else if s.contains("yoga") || s.contains("asana") || s.contains("poses") {
        Some("yoga")
    } else if s.contains("weight") || s.contains("strength") || s.contains("gym") || s.contains("lifting") {
        Some("strength")
    } else if s.contains("exercise") || s.contains("workout") {
        None
    } else {
        Some("")
    }
}

fn card(e: &'static Ex, focus: Option<&str>) -> Answer {
    let name = e.names[0];
    let other = e.names[1..].iter().filter(|n| n.chars().next().is_some_and(|c| c.is_uppercase())).copied().collect::<Vec<_>>();
    let muscles = if e.muscles.is_empty() { "—".to_string() } else { e.muscles.join(", ") };
    let (title, value) = match focus {
        Some("muscles") => ("Muscles worked", muscles.clone()),
        Some("how") => ("How to do it", e.how.to_string()),
        Some("benefits") => ("Benefits", e.benefits.to_string()),
        _ if e.category == "sequence" => ("Sequence", e.how.to_string()),
        _ => ("Result", format!("{} {} ({}); works {}", e.kind, e.category, e.level, muscles)),
    };
    let mut rows = vec![vec![text("type"), text(format!("{} {}", e.kind, e.category))], vec![text("level"), text(e.level)]];
    if !other.is_empty() {
        rows.push(vec![text("other names"), text(other.join(", "))]);
    }
    if !e.muscles.is_empty() {
        rows.push(vec![text("muscles worked"), text(muscles)]);
    }
    rows.push(vec![text("benefits"), text(e.benefits)]);
    let mut pods = vec![pod("Input", "Input interpretation", text(format!("{name} ({} {})", e.kind, e.category))), primary(pod("Result", title, text(value))), pod("Properties", "Properties", table(None, rows))];
    if title != "How to do it" && title != "Sequence" {
        pods.push(pod("How", if e.category == "sequence" { "Sequence" } else { "How to do it" }, text(e.how)));
    }
    pods.push(pod("Note", "Note", text("general guidance, not medical advice; warm up first and stop if anything hurts")));
    Answer::new(name, pods)
}

fn listing(title: String, items: Vec<&'static Ex>) -> Option<Answer> {
    if items.is_empty() {
        return None;
    }
    let short = items.iter().map(|e| e.names[0]).collect::<Vec<_>>().join(" | ");
    let rows = items.iter().map(|e| vec![text(e.names[0]), text(format!("{} {}", e.kind, e.category)), text(e.level), text(e.muscles.join(", "))]).collect();
    Some(Answer::new("exercises", vec![
        pod("Input", "Input interpretation", text(title)),
        primary(pod("Result", "Exercises", text(short))),
        pod("Table", "Details", table(Some(vec!["exercise", "type", "level", "muscles worked"]), rows)),
    ]))
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.as_str();
    // "what muscles does a plank work", "hundred pilates primary contracted muscles",
    // "what muscles should I engage to support side bend"
    if let Some(m) = re!(r"^(?:what|which) muscles? (?:does|do|are used in|are worked by|are used by|should i engage to support|help support|support|are engaged in) (?:a |an |the )?(.+?)(?: work| use| target| engage)?$|^(?:muscles? (?:worked|used|engaged|targeted) (?:by|in|during) )(?:a |an |the )?(.+)$|^(.+?) (?:primary |main )?(?:contracted |worked |targeted |engaged )?muscles(?: worked| used)?$").captures(l) {
        let s = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
        let s = s.trim_end_matches(" primary").trim_end_matches(" main");
        if let Some(e) = find(s) {
            return Some(card(e, Some("muscles")));
        }
    }
    // "how do you do a squat", "how to do the hundred", "squat technique"
    if let Some(m) = re!(r"^how (?:do|can|should) (?:you|i|one|we) (?:do|perform|practice|get into) (?:a |an |the )?(.+)$|^how to (?:do|perform|practice|get into) (?:a |an |the )?(.+)$|^(.+?) (?:technique|form|instructions|steps)$").captures(l) {
        let s = m.get(1).or(m.get(2)).or(m.get(3))?.as_str();
        if let Some(e) = find(s) {
            return Some(card(e, Some("how")));
        }
    }
    // "benefits of pigeon pose"
    if let Some(m) = re!(r"^(?:benefits|uses) of (?:a |an |the )?(.+)$|^(.+?) benefits$").captures(l) {
        let s = m.get(1).or(m.get(2))?.as_str();
        if let Some(e) = find(s) {
            return Some(card(e, Some("benefits")));
        }
    }
    // "yoga poses to strengthen abdominal muscles", "exercises for the glutes",
    // "Pilates exercises for better posture"
    if let Some(m) = re!(r"^(?:best |good )?((?:[a-z]+ )?(?:yoga poses|yoga|poses|asanas|pilates exercises|pilates|exercises|workouts|moves|weight training exercises|strength exercises)) (?:to (?:strengthen|work|build|tone|target|stretch|improve|help|open)|for|that (?:strengthen|work|target|improve)) (?:the |your |my |better |stronger )?(.+)$").captures(l) {
        let kind = kind_of(&m[1]);
        let target = m[2].trim();
        let kind_ok = |e: &&Ex| kind.map_or(true, |k| k.is_empty() || e.kind == k) && e.category != "sequence";
        let items: Vec<&'static Ex> = if let Some(ms) = muscle(target) {
            // the exercises that list the muscle first work it most
            let mut v: Vec<(usize, &'static Ex)> = all().iter().filter(kind_ok).filter_map(|e| e.muscles.iter().position(|x| ms.contains(x)).map(|k| (k, e))).collect();
            v.sort_by_key(|(k, _)| *k);
            v.into_iter().map(|(_, e)| e).collect()
        } else {
            let t = target.trim_end_matches(" muscles");
            let w = t.split_whitespace().last().unwrap_or(t);
            all().iter().filter(kind_ok).filter(|e| e.benefits.contains(w)).collect()
        };
        return listing(format!("{} for {target}", m[1].trim()), items.into_iter().take(25).collect());
    }
    // "advanced Pilates exercises", "beginner yoga poses", "isolated weight training exercises",
    // "pilates exercises", "standing yoga poses", "yoga backbends", "arm balances"
    if let Some(m) = re!(r"^(?:list of |types of )?(beginner|easy|basic|intermediate|advanced|hard|difficult|isolated|isolation|compound|standing|seated|balancing|balance|inversion|backbend|twisting|resting|hip opening|classical|classic|mat)?(?: level)? ?(yoga poses|yoga postures|yoga asanas|asanas|yoga|pilates exercises|pilates mat exercises|pilates moves|pilates|weight training exercises|weightlifting exercises|strength training exercises|strength exercises|gym exercises|exercises|yoga backbends|yoga inversions|arm balances|yoga sequences)$").captures(l) {
        let adj = m.get(1).map(|x| x.as_str());
        let k = &m[2];
        let kind = kind_of(k);
        if adj.is_none() && kind.is_none() {
            return None;
        }
        let items: Vec<&'static Ex> = all()
            .iter()
            .filter(|e| match kind {
                Some("") | None => true,
                Some(k) => e.kind == k,
            })
            .filter(|e| match k {
                "yoga backbends" => e.category == "backbend",
                "yoga inversions" => e.category == "inversion",
                "arm balances" => e.category == "arm balance",
                "yoga sequences" => e.category == "sequence",
                _ => true,
            })
            .filter(|e| match adj {
                Some("beginner" | "easy" | "basic") => e.level == "beginner",
                Some("intermediate") => e.level == "intermediate",
                Some("advanced" | "hard" | "difficult") => e.level == "advanced",
                Some("isolated" | "isolation") => e.category == "isolation",
                Some("compound") => e.category == "compound",
                Some("standing") => e.category == "standing pose",
                Some("seated") => e.category == "seated pose",
                Some("balancing" | "balance") => e.category == "balancing pose",
                Some("inversion") => e.category == "inversion",
                Some("backbend") => e.category == "backbend",
                Some("twisting") => e.category == "twist",
                Some("resting") => e.category == "resting pose",
                Some("hip opening") => e.category == "hip opener",
                _ => true,
            })
            .collect();
        return listing(l.to_string(), items);
    }
    // "downward facing dog, upward facing dog pose"
    let parts: Vec<&str> = re!(r"\s*(?:,|\band\b|\bvs\.?(?:\s|$)|\bversus\b)\s*").split(l).filter(|x| !x.trim().is_empty()).collect();
    if parts.len() >= 2 {
        if let Some(es) = parts.iter().map(|p| find(p)).collect::<Option<Vec<&Ex>>>() {
            let rows = es.iter().map(|e| vec![text(e.names[0]), text(e.kind), text(e.category), text(e.level), text(e.muscles.join(", "))]).collect();
            return Some(Answer::new("exercises", vec![
                pod("Input", "Input interpretation", text(es.iter().map(|e| e.names[0]).collect::<Vec<_>>().join(" | "))),
                primary(pod("Result", "Comparison", text(es.iter().map(|e| format!("{}: {} {}, works {}", e.names[0], e.kind, e.category, e.muscles.join(", "))).collect::<Vec<_>>().join(" | ")))),
                pod("Table", "Details", table(Some(vec!["exercise", "discipline", "category", "level", "muscles"]), rows)),
            ]));
        }
    }
    // a bare exercise name: "Pincha Mayurasana", "double leg stretch", "Sun Salutation A"
    let e = find(l)?;
    let generic = ["squat", "squats", "bench", "row", "dip", "dips", "swimming", "bicycle", "crab", "rocking", "seal", "saw", "the saw", "swan", "teaser", "corkscrew", "jackknife", "scissors", "mermaid", "shrug", "shrugs", "curls", "hundred", "hundreds", "bridge", "tree", "chair", "cobra", "camel", "bow", "boat", "crow", "lotus", "pigeon", "triangle", "mountain", "eagle", "dancer", "locust", "corpse", "plank", "lunge", "lunges", "burpee", "ashtanga", "wheel", "cat"];
    if generic.contains(&l) || !e.bare {
        // a common word on its own: only exercise names that are clearly exercises
        if !["squat", "squats", "plank", "lunge", "lunges", "burpee", "ashtanga", "hundred"].contains(&l) {
            return None;
        }
    }
    Some(card(e, None))
}
