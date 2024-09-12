use crate::imports::*;

const MAX_AVERAGE_SAMPLES: usize = 6;
const AVERAGE_ALPHA_HIGH: f64 = 0.8;
const AVERAGE_ALPHA_LOW: f64 = 0.5;

#[derive(Default)]
pub struct FeerateEstimate {
    pub low: FeerateBucketAverage,
    pub economic: FeerateBucketAverage,
    pub priority: FeerateBucketAverage,
}

// impl Default for FeerateEstimate {
//     fn default() -> Self {
//         Self {
//             low: FeerateBucketAverage::default(),
//             economic: FeerateBucketAverage::default(),
//             priority: FeerateBucketAverage::default(),
//         }
//     }
// }

impl FeerateEstimate {
    pub fn new(estimate: &RpcFeeEstimate) -> Self {
        let mut feerate = Self {
            low: FeerateBucketAverage::default(),
            economic: FeerateBucketAverage::default(),
            priority: FeerateBucketAverage::default(),
        };
        feerate.insert(estimate);
        feerate
    }

    pub fn insert(&mut self, estimate: &RpcFeeEstimate) {
        self.low.insert(
            estimate
                .low_buckets
                .first()
                .map(FeerateBucket::from)
                .unwrap_or_default(),
        );
        self.economic.insert(
            estimate
                .normal_buckets
                .first()
                .map(FeerateBucket::from)
                .unwrap_or_default(),
        );
        self.priority.insert(estimate.priority_bucket.into());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FeerateBucket {
    pub feerate: f64,
    pub seconds: f64,
}

impl FeerateBucket {
    pub fn new(feerate: f64, seconds: f64) -> Self {
        Self { feerate, seconds }
    }

    pub fn with_seconds(self, seconds: f64) -> Self {
        Self {
            feerate: self.feerate,
            seconds,
        }
    }
}

impl std::cmp::PartialOrd for FeerateBucket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.feerate.partial_cmp(&other.feerate)
    }
}

impl std::cmp::PartialEq for FeerateBucket {
    fn eq(&self, other: &Self) -> bool {
        self.feerate == other.feerate
    }
}

impl std::ops::Add for FeerateBucket {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            feerate: self.feerate + other.feerate,
            seconds: self.seconds + other.seconds,
        }
    }
}

impl Default for FeerateBucket {
    fn default() -> Self {
        Self {
            feerate: 1.0,
            seconds: 1.0,
        }
    }
}

impl From<&RpcFeerateBucket> for FeerateBucket {
    fn from(bucket: &RpcFeerateBucket) -> Self {
        Self {
            feerate: bucket.feerate,
            seconds: bucket.estimated_seconds,
        }
    }
}

impl From<RpcFeerateBucket> for FeerateBucket {
    fn from(bucket: RpcFeerateBucket) -> Self {
        Self {
            feerate: bucket.feerate,
            seconds: bucket.estimated_seconds,
        }
    }
}

pub type FeerateBucketAverage = FeerateBucketAverageN<MAX_AVERAGE_SAMPLES>;

#[derive(Default, Debug, Clone)]
pub struct FeerateBucketAverageN<const SAMPLES: usize> {
    pub samples: VecDeque<FeerateBucket>,
    pub average: FeerateBucket,
}

impl<const SAMPLES: usize> FeerateBucketAverageN<SAMPLES> {
    pub fn clear(&mut self) {
        self.samples.clear();
        self.average = FeerateBucket::default();
    }

    fn insert(&mut self, value: FeerateBucket) {
        if self.samples.is_empty() {
            self.samples.push_back(value);
        } else {
            let delta = self.average;

            if value > self.value() {
                let feerate =
                    AVERAGE_ALPHA_HIGH * value.feerate + (1.0 - AVERAGE_ALPHA_HIGH) * delta.feerate;
                let seconds =
                    AVERAGE_ALPHA_HIGH * value.seconds + (1.0 - AVERAGE_ALPHA_HIGH) * delta.seconds;
                self.samples.push_back(FeerateBucket::new(feerate, seconds));
            } else {
                let feerate =
                    AVERAGE_ALPHA_LOW * value.feerate + (1.0 - AVERAGE_ALPHA_LOW) * delta.feerate;
                let seconds =
                    AVERAGE_ALPHA_LOW * value.seconds + (1.0 - AVERAGE_ALPHA_LOW) * delta.seconds;
                self.samples.push_back(FeerateBucket::new(feerate, seconds));
            }
        }

        if self.samples.len() > SAMPLES {
            self.samples.pop_front();
        }
        self.update();
    }

    pub fn update(&mut self) {
        let len = self.samples.len() as f64;
        let sum = self
            .samples
            .iter()
            .fold(FeerateBucket::default(), |a, b| a + *b);
        self.average = FeerateBucket {
            feerate: sum.feerate / len,
            seconds: sum.seconds / len,
        };
    }

    pub fn value(&self) -> FeerateBucket {
        if let Some(value) = self.samples.back() {
            *value
        } else {
            FeerateBucket::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_average() {
        let values = [
            248155.3514557079,
            215263.2471686213,
            245860.5071433737,
            182878.35543534035,
            247603.8863732086,
            192076.06581364156,
            191832.4918487833,
            128304.96588496015,
            238531.64457851378,
            234273.18428987174,
            255773.0300519179,
            249093.50547463333,
            249987.27849438024,
            142243.6036012581,
            177387.34711973823,
            225326.88080792018,
            246519.9389768766,
            178864.37118321584,
            153275.66505913227,
            246842.79445636555,
            223232.92238098633,
            224018.9368980746,
            144846.07860895086,
            249044.80135444857,
            193644.7196123328,
            216569.54469648135,
            171501.87818606084,
            209143.00683348018,
            247766.34940556984,
            240134.74496231132,
            222067.39937692153,
            244959.2666668208,
            244778.55586060378,
            236770.12079389597,
            150160.29996444215,
            245368.49577914673,
            237944.53312366523,
            233158.65408234956,
            133382.8086967139,
            236848.24272788866,
            144236.5336527448,
            148653.23967874967,
            238307.96108361648,
            169613.28962313317,
            238433.97536641933,
            177764.92055346936,
            139490.52495366355,
            187191.81261710104,
            244475.82286127112,
            245749.91495551568,
            243281.21882477857,
            243326.62347613362,
            196588.33586232664,
            120549.67729575293,
            207835.44585471656,
            200133.85199050562,
            151756.37033541917,
            235157.25117139242,
            246507.34219541284,
            176741.3115867982,
            99382.172497149,
            187744.75694246456,
            232886.15175851452,
            175217.07336267238,
            236816.7528010096,
            139970.4400116053,
            159865.71770829588,
            136558.44865940727,
            195715.8517624622,
            247389.84374880078,
            239440.79812678922,
            135748.0474010845,
            210934.440126234,
            146065.89308193082,
            147800.1502998639,
            244894.88053721157,
            243994.97948386057,
            245393.49415038983,
            243058.58821456484,
            127735.35987305158,
            140369.60399027826,
            134043.303765809,
            193254.1420857881,
            190437.9472722749,
            244581.3539859155,
            245565.95352441736,
            146978.29638989634,
            245169.43506911577,
            244653.8568352576,
            219344.22021480868,
            237979.00370085222,
            187110.5903983477,
            241250.8277014743,
            233906.6233636831,
            233246.14065750796,
            233413.1716293142,
            233148.15345403645,
            235678.72315910575,
            229063.60353987105,
            240832.9716861154,
            247508.69135150572,
            168210.11234128906,
            210847.66573606082,
            246857.0640787107,
            246239.95443353,
            171438.46158665072,
            247079.0881248516,
            171697.12570214068,
            149564.49850670493,
            245807.68863258162,
            149489.9342934734,
            242506.84793606773,
            242075.0260494275,
            220151.7465220994,
            126044.54038402438,
            243012.56791447278,
            241111.42157753845,
            235986.34918704405,
            143356.96079931365,
            130424.4130526057,
            181255.2246665847,
            238256.8730829127,
            146452.11754679924,
            170026.6602172315,
            193312.99575573037,
            239332.9420721534,
            223632.38567019213,
            141671.20874054675,
            213208.87877292512,
            219982.01542181577,
            134341.80571515945,
            194831.33549576855,
            239883.47734007097,
            88967.05109533986,
            202053.81995419698,
            243547.28828147077,
            259553.12840624293,
            247677.92768900306,
            242653.40883194323,
            174398.6326048576,
            128285.83328560732,
            244399.46198033314,
            146487.26658813033,
            241494.59142860002,
            240616.70796003094,
            138833.95391005423,
            213584.339225704,
            239133.53643788848,
            237835.51801700686,
            237347.47389371862,
            234469.64315771483,
            152160.9061874391,
            237494.77127650633,
            173040.8121341394,
            130912.77797708409,
            241962.979458208,
            182373.4185260581,
            179541.5825536582,
            179029.96761306134,
            243205.50735454418,
            199721.08821708092,
            259852.2696898588,
            228419.10949632138,
            248855.05259311185,
            150812.03274903848,
            147457.01306234082,
            244726.97247294002,
            146187.5743984257,
            244823.66242046928,
            244862.57860557683,
            244955.87378775087,
            130421.35339233676,
            244445.4788989109,
            137780.17782492848,
            133758.41619601438,
            156704.98698541842,
            238910.814299598,
            240336.87067486043,
            214522.15232621093,
            130056.62527973427,
            178634.3844432882,
            244888.6989210272,
            217647.4456866496,
            244594.37186805526,
            228030.16662885746,
            214253.44980608378,
            209516.1694079057,
            244868.00533888358,
            233448.87699662437,
            257545.7255320017,
            249172.88737132723,
            232243.2726875817,
            136311.39666593345,
            249623.2939563043,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            215391.32088258484,
            67.52105424968201,
            242234.10627665027,
            168573.38189513536,
            231680.36189481552,
            207704.39080488606,
            248347.3469480634,
            211414.393660134,
            248765.77597833515,
            84283.68751575101,
            239052.37494681875,
            175256.67273241273,
            127825.05894308779,
            216616.31935138846,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            238537.57586429152,
            127547.14902881409,
            135879.29173228334,
            238302.0978156781,
            135926.23165511477,
            246619.3074482109,
            248641.3156527779,
            226314.28646016173,
            154861.34389305062,
            204459.3146002139,
            181339.61465791994,
            246846.6580346392,
            207506.03970416822,
            199709.8717643581,
            178158.12966541818,
            246227.38389916637,
            141832.6854908764,
            237995.89026512875,
            245124.70220074043,
            245836.33998509325,
            247250.12234223183,
            244902.619869532,
            251890.4761664218,
            252799.9852909657,
            253275.67723549923,
            241236.4859757893,
            238396.0150783782,
            183664.4147463932,
            242305.507166485,
            181229.56382171725,
            242067.5188488911,
            250022.01325817694,
            179025.21296337966,
            131457.8297950491,
            177872.42410698708,
            245104.4716619338,
            243458.21421007247,
            245401.3834330709,
            205715.71134339966,
            244804.82748832856,
            244561.01289969,
            157427.76485069326,
            186983.8334490723,
            231963.53189224305,
            172578.63845091636,
            245203.00423633948,
            151143.67479554564,
            245177.82270082034,
            246292.37314888765,
            68855.01652778508,
            246382.54972762943,
            245977.19630932037,
            245196.2241682157,
            194767.96677656696,
            245693.9823006708,
            242633.14689425754,
            242418.49831178205,
            242756.2666038934,
            243668.81635819998,
            0.0,
            245422.2838396944,
            244682.78711597863,
            244792.26394066506,
            152821.9161276334,
            249688.58018299768,
            249301.2771748242,
            181230.20839436172,
            257962.65317966897,
            248043.1062423161,
            258443.0871438949,
            224195.3353968486,
            161304.520674059,
            256466.31079941467,
            257594.97335507823,
            188122.8387603218,
            256452.86741962022,
            158010.11777856835,
            151733.4649294484,
            246619.54674299472,
            152716.32651794696,
        ];

        let mut average = FeerateBucketAverageN::<6>::default();
        for value in values.iter() {
            average.insert(FeerateBucket::new(*value, 1.0));
            println!("{} -> {:?}", value, average.value());
        }
    }
}
