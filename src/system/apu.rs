use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type};

use crate::cartridge::Mapper;

use super::apu_util::{
    DmcChannel, NesChannel, NoiseChannel, PulseChannel, TriangleChannel
};

pub const NES_AUDIO_FREQUENCY: u32 = 44100; // 44.1 KiHz
pub const CPU_FREQ: f64 = 1_789_773f64; // For NTSC systems
pub const CPU_CYCLE_PERIOD: f64 = 1.0 / CPU_FREQ;

const SAMPLE_PERIOD: f64 = 1.0 / NES_AUDIO_FREQUENCY as f64;
const SAMPLE_BATCH_SIZE: usize = 2048;
// The number of clocks in each denomination of a frame (in CPU clocks)
const QUARTER_FRAME_CLOCKS: usize = 3729;
const HALF_FRAME_CLOCKS: usize = 7457;
const THREE_QUARTER_FRAME_CLOCKS: usize = 11185;
const WHOLE_FRAME_CLOCKS: usize = 14916;

// Cutoff frequencies of the filters given in Hz
const HIGH_PASS1_CUTOFF_FREQ: f32 = 60.0;
const HIGH_PASS2_CUTOFF_FREQ: f32 = 440.0;
const LOW_PASS_CUTOFF_FREQ: f32 = 14000.0;

// Quality factor ( 1/sqrt(2) is customary )
const Q_VAL: f32 = 0.7071067811865475244008443622;

pub struct Apu2A03 {
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    sample_batch: Vec<f32>,

    mapper: Rc<RefCell<dyn Mapper>>,
    
    clocks: u64,
    frame_clocks: usize,
    clocks_since_sampled: usize,

    pulse1_channel: PulseChannel,
    pulse2_channel: PulseChannel,
    triangle_channel: TriangleChannel,
    noise_channel: NoiseChannel,
    dmc_channel: DmcChannel,

    high_pass1: DirectForm1<f32>,
    high_pass2: DirectForm1<f32>,
    low_pass: DirectForm1<f32>,

    frame_sequence: bool,
    disable_frame_interrupt: bool,

    frame_update_counter: usize,
    frame_update_mode1: bool,

    last_output: Instant,
    batches_sent: usize,

    irq_request_flag: bool,
    trigger_irq: bool,
}

impl Apu2A03 {
    // Sample lookup table for the pulse wave channels. All terms calculated using the formula
    // PULSE_LOOKUP[n] = 95.52 / (8128.0 / n + 100)
    // indexed as PULSE_LOOKUP[ pulse1 + pulse2 ] (max values of 15 + 15 == 30)
    // from the nesdev wiki: https://www.nesdev.org/wiki/APU_Mixer
    const PULSE_LOOKUP: [f32; 31] = [
        1.175196850393700787401574803, 1.186948818897637795275590551, 1.198700787401574803149606299, 1.210452755905511811023622047, 1.222204724409448818897637795, 1.233956692913385826771653543, 1.245708661417322834645669291, 1.257460629921259842519685039,
        1.269212598425196850393700787, 1.280964566929133858267716536, 1.292716535433070866141732283, 1.304468503937007874015748031, 1.316220472440944881889763780, 1.327972440944881889763779528, 1.339724409448818897637795276, 1.351476377952755905511811024,
        1.363228346456692913385826772, 1.374980314960629921259842520, 1.386732283464566929133858268, 1.398484251968503937007874016, 1.410236220472440944881889764, 1.421988188976377952755905512, 1.433740157480314960629921260, 1.445492125984251968503937008,
        1.457244094488188976377952756, 1.468996062992125984251968504, 1.480748031496062992125984252, 1.492500000000000000000000000, 1.504251968503937007874015748, 1.516003937007874015748031496, 1.527755905511811023622047244, 
    ];

    // Sample lookup table for the DMC + Triangle + Noise channels. All terms calculated using the formula
    // TND_LOOKUP[n] = 163.67 / (24329.0 / n + 100)
    // indexed as TND_LOOKUP[ 3*triangle + 2*noise + dmc ] (max values of 3*15 + 2*15 + 128 == 203)
    // from the nesdev wiki: https://www.nesdev.org/wiki/APU_Mixer
    const TND_LOOKUP: [f32; 204] = [
        0.672736240700398701138558922, 0.679463603107402688149944511, 0.686190965514406675161330100, 0.692918327921410662172715689, 0.699645690328414649184101278, 0.706373052735418636195486868, 0.713100415142422623206872457, 0.719827777549426610218258046,
        0.726555139956430597229643635, 0.733282502363434584241029224, 0.740009864770438571252414814, 0.746737227177442558263800403, 0.753464589584446545275185992, 0.760191951991450532286571581, 0.766919314398454519297957170, 0.773646676805458506309342760,
        0.780374039212462493320728349, 0.787101401619466480332113938, 0.793828764026470467343499528, 0.800556126433474454354885116, 0.807283488840478441366270706, 0.814010851247482428377656295, 0.820738213654486415389041884, 0.827465576061490402400427473,
        0.834192938468494389411813063, 0.840920300875498376423198652, 0.847647663282502363434584241, 0.854375025689506350445969830, 0.861102388096510337457355420, 0.867829750503514324468741009, 0.874557112910518311480126598, 0.881284475317522298491512187,
        0.888011837724526285502897776, 0.894739200131530272514283366, 0.901466562538534259525668955, 0.908193924945538246537054544, 0.914921287352542233548440133, 0.921648649759546220559825722, 0.928376012166550207571211312, 0.935103374573554194582596901,
        0.941830736980558181593982490, 0.948558099387562168605368079, 0.955285461794566155616753669, 0.962012824201570142628139258, 0.968740186608574129639524847, 0.975467549015578116650910436, 0.982194911422582103662296026, 0.988922273829586090673681615,
        0.995649636236590077685067204, 1.002376998643594064696452793, 1.009104361050598051707838382, 1.015831723457602038719223971, 1.022559085864606025730609561, 1.029286448271610012741995150, 1.036013810678613999753380739, 1.042741173085617986764766328,
        1.049468535492621973776151917, 1.056195897899625960787537506, 1.062923260306629947798923096, 1.069650622713633934810308685, 1.076377985120637921821694274, 1.083105347527641908833079863, 1.089832709934645895844465453, 1.096560072341649882855851042,
        1.103287434748653869867236631, 1.110014797155657856878622221, 1.116742159562661843890007809, 1.123469521969665830901393398, 1.130196884376669817912778988, 1.136924246783673804924164577, 1.143651609190677791935550166, 1.150378971597681778946935756,
        1.157106334004685765958321345, 1.163833696411689752969706934, 1.170561058818693739981092524, 1.177288421225697726992478113, 1.184015783632701714003863702, 1.190743146039705701015249291, 1.197470508446709688026634880, 1.204197870853713675038020470,
        1.210925233260717662049406059, 1.217652595667721649060791648, 1.224379958074725636072177237, 1.231107320481729623083562826, 1.237834682888733610094948415, 1.244562045295737597106334005, 1.251289407702741584117719594, 1.258016770109745571129105183,
        1.264744132516749558140490772, 1.271471494923753545151876362, 1.278198857330757532163261951, 1.284926219737761519174647540, 1.291653582144765506186033129, 1.298380944551769493197418718, 1.305108306958773480208804308, 1.311835669365777467220189896,
        1.318563031772781454231575487, 1.325290394179785441242961075, 1.332017756586789428254346664, 1.338745118993793415265732254, 1.345472481400797402277117843, 1.352199843807801389288503432, 1.358927206214805376299889022, 1.365654568621809363311274611,
        1.372381931028813350322660199, 1.379109293435817337334045789, 1.385836655842821324345431378, 1.392564018249825311356816968, 1.399291380656829298368202556, 1.406018743063833285379588145, 1.412746105470837272390973734, 1.419473467877841259402359324,
        1.426200830284845246413744914, 1.432928192691849233425130503, 1.439655555098853220436516092, 1.446382917505857207447901682, 1.453110279912861194459287270, 1.459837642319865181470672859, 1.466565004726869168482058449, 1.473292367133873155493444038,
        1.480019729540877142504829628, 1.486747091947881129516215217, 1.493474454354885116527600805, 1.500201816761889103538986395, 1.506929179168893090550371985, 1.513656541575897077561757573, 1.520383903982901064573143162, 1.527111266389905051584528751,
        1.533838628796909038595914340, 1.540565991203913025607299930, 1.547293353610917012618685520, 1.554020716017920999630071109, 1.560748078424924986641456697, 1.567475440831928973652842287, 1.574202803238932960664227876, 1.580930165645936947675613466,
        1.587657528052940934686999054, 1.594384890459944921698384643, 1.601112252866948908709770234, 1.607839615273952895721155822, 1.614566977680956882732541412, 1.621294340087960869743927001, 1.628021702494964856755312590, 1.634749064901968843766698179,
        1.641476427308972830778083768, 1.648203789715976817789469358, 1.654931152122980804800854947, 1.661658514529984791812240536, 1.668385876936988778823626125, 1.675113239343992765835011714, 1.681840601750996752846397304, 1.688567964158000739857782893,
        1.695295326565004726869168482, 1.702022688972008713880554071, 1.708750051379012700891939660, 1.715477413786016687903325250, 1.722204776193020674914710839, 1.728932138600024661926096428, 1.735659501007028648937482017, 1.742386863414032635948867607,
        1.749114225821036622960253196, 1.755841588228040609971638785, 1.762568950635044596983024374, 1.769296313042048583994409963, 1.776023675449052571005795553, 1.782751037856056558017181142, 1.789478400263060545028566731, 1.796205762670064532039952320,
        1.802933125077068519051337909, 1.809660487484072506062723499, 1.816387849891076493074109088, 1.823115212298080480085494677, 1.829842574705084467096880266, 1.836569937112088454108265856, 1.843297299519092441119651445, 1.850024661926096428131037034,
        1.856752024333100415142422623, 1.863479386740104402153808212, 1.870206749147108389165193802, 1.876934111554112376176579391, 1.883661473961116363187964980, 1.890388836368120350199350569, 1.897116198775124337210736159, 1.903843561182128324222121748,
        1.910570923589132311233507337, 1.917298285996136298244892926, 1.924025648403140285256278515, 1.930753010810144272267664105, 1.937480373217148259279049694, 1.944207735624152246290435283, 1.950935098031156233301820872, 1.957662460438160220313206461,
        1.964389822845164207324592051, 1.971117185252168194335977640, 1.977844547659172181347363229, 1.984571910066176168358748818, 1.991299272473180155370134407, 1.998026634880184142381519997, 2.004753997287188129392905586, 2.011481359694192116404291175,
        2.018208722101196103415676764, 2.024936084508200090427062353, 2.031663446915204077438447943, 2.038390809322208064449833532,
    ];


    pub fn new(sample_queue: Arc<Mutex<VecDeque<f32>>>, mapper: Rc<RefCell<dyn Mapper>>) -> Self {
        let high_pass1_coeffs: Coefficients<f32> = Coefficients::<f32>::from_params(
            Type::HighPass,
            NES_AUDIO_FREQUENCY.hz(),
            HIGH_PASS1_CUTOFF_FREQ.hz(),
            Q_VAL,
        ).unwrap();
    
        let high_pass2_coeffs: Coefficients<f32> = Coefficients::<f32>::from_params(
            Type::HighPass,
            NES_AUDIO_FREQUENCY.hz(),
            HIGH_PASS2_CUTOFF_FREQ.hz(),
            Q_VAL,
        ).unwrap();
    
        let low_pass_coeffs: Coefficients<f32> = Coefficients::<f32>::from_params(
            Type::LowPass,
            NES_AUDIO_FREQUENCY.hz(),
            LOW_PASS_CUTOFF_FREQ.hz(),
            Q_VAL,
        ).unwrap();

        Self {
            sample_queue,
            sample_batch: Vec::with_capacity(SAMPLE_BATCH_SIZE),

            mapper,

            clocks: 0,
            frame_clocks: 0,
            clocks_since_sampled: 0,

            pulse1_channel: PulseChannel::new(NesChannel::Pulse1),
            pulse2_channel: PulseChannel::new(NesChannel::Pulse2),
            triangle_channel: TriangleChannel::default(),
            noise_channel: NoiseChannel::new(),
            dmc_channel: DmcChannel::new(),

            high_pass1: DirectForm1::<f32>::new(high_pass1_coeffs),
            high_pass2: DirectForm1::<f32>::new(high_pass2_coeffs),
            low_pass: DirectForm1::<f32>::new(low_pass_coeffs),

            frame_sequence: false,
            disable_frame_interrupt: false,

            frame_update_counter: 0,
            frame_update_mode1: false,

            last_output: Instant::now(),
            batches_sent: 0,

            irq_request_flag: false,
            trigger_irq: false,
        }
    }

    pub fn cycle(&mut self) {
        self.clocks += 1;
        self.frame_clocks += 1;
        self.clocks_since_sampled += 1;

        // Noise channel updates period every CPU clock
        self.noise_channel.update_period();
        // DMC channel updates its timer every CPU clock
        if self.dmc_channel.need_next_clip_byte() {
            let addr = self.dmc_channel.current_sample_addr();

            // Audio clips start at $C000, which should always be 
            // accessing valid cartridge memory.
            let next_clip_byte = self.mapper.as_ref()
                                                .borrow_mut()
                                                .cpu_cart_read(addr)
                                                .unwrap();

            self.dmc_channel.update_timer(Some(next_clip_byte));

        } else {
            self.dmc_channel.update_timer(None);
        }

        if self.frame_clocks == QUARTER_FRAME_CLOCKS
            || self.frame_clocks == HALF_FRAME_CLOCKS
            || self.frame_clocks == THREE_QUARTER_FRAME_CLOCKS
            || self.frame_clocks == WHOLE_FRAME_CLOCKS {
            
            self.frame_update();
        
            if self.frame_clocks == WHOLE_FRAME_CLOCKS {
                self.frame_clocks = 0;
            }
        }

        let time_since_sampled = self.clocks_since_sampled as f64 * CPU_CYCLE_PERIOD;

        if time_since_sampled > SAMPLE_PERIOD {
            let sample = self.generate_sample();

            self.push_sample(sample);

            self.clocks_since_sampled = 0;
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        match address {
            0x4015 => {
                // DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1) 
                let d = if self.dmc_channel.dmc_active() { 1 } else { 0 };
                let n = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let t = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let p1 = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };
                let p2 = if self.pulse1_channel.length_counter.is_zero() { 0 } else { 1 };

                let data = (d << 4) | (n << 3) | (t << 2) | (p2 << 1) | (p1 << 0);

                data as u8
            },

            _ => 0
        }
    }

    pub fn cpu_write(&mut self, address: u16, data: u8) {
        match address {
            // Pulse 1 Registers
            0x4000 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_halt = (data & 0x20) != 0; // Also envelope's loop flag
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                self.pulse1_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };
                self.pulse1_channel.length_counter.set_halted(new_halt);
                self.pulse1_channel.envelope.set_loop_flag(new_halt);
                self.pulse1_channel.envelope.set_const_volume(new_const_volume);
                self.pulse1_channel.envelope.set_volume(new_volume);
                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 1 Sweeper
            0x4001 => {
                let new_enable = (data & 0x80) != 0;
                let new_reload_val = ((data >> 4) & 7) as usize;
                let new_negate = (data & 0x08) != 0;
                let new_shift = (data & 7) as usize;

                self.pulse1_channel.set_sweep_enable(new_enable);
                self.pulse1_channel.set_sweep_period(new_reload_val);
                self.pulse1_channel.set_sweep_negate_flag(new_negate);
                self.pulse1_channel.set_sweep_shift(new_shift);
                self.pulse1_channel.set_sweep_reload_flag(true);
                self.pulse1_channel.update_target_period();
            }

            // Pulse 1 Timer Low
            0x4002 => {
                self.pulse1_channel.set_timer_reload(
                    (self.pulse1_channel.timer_reload() & 0x700) | data as usize
                );

                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 1 Timer High & Length Counter
            0x4003 => {
                let new_counter_load = (data >> 3) as usize;
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.pulse1_channel.timer_reload() & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;

                self.pulse1_channel.length_counter.set_counter(new_counter_load);
                self.pulse1_channel.set_timer_reload(new_timer);
                self.pulse1_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Registers
            0x4004 => {
                let new_duty_cycle = (data >> 6) & 3;
                let new_halt = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                self.pulse2_channel.duty_cycle_percent = match new_duty_cycle {
                    0 => 0.125,
                    1 => 0.25,
                    2 => 0.50,
                    3 => 0.75,
                    _ => { unreachable!("Holy wack unlyrical lyrics, Batman!"); }
                };
                self.pulse2_channel.length_counter.set_halted(new_halt);
                self.pulse2_channel.envelope.set_loop_flag(new_halt);
                self.pulse2_channel.envelope.set_const_volume(new_const_volume);
                self.pulse2_channel.envelope.set_volume(new_volume);
                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Sweeper
            0x4005 => {
                let new_enable = (data & 0x80) != 0;
                let new_reload_val = ((data >> 4) & 7) as usize;
                let new_negate = (data & 0x08) != 0;
                let new_shift = (data & 7) as usize;

                self.pulse2_channel.set_sweep_enable(new_enable);
                self.pulse2_channel.set_sweep_period(new_reload_val);
                self.pulse2_channel.set_sweep_negate_flag(new_negate);
                self.pulse2_channel.set_sweep_shift(new_shift);
                self.pulse2_channel.set_sweep_reload_flag(true);
                self.pulse2_channel.update_target_period();
            }

            // Pulse 2 Timer Low
            0x4006 => {
                self.pulse2_channel.set_timer_reload(
                    (self.pulse2_channel.timer_reload() & 0x700) | data as usize
                );

                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Pulse 2 Timer High & Length Counter
            0x4007 => {
                let new_counter_load = (data >> 3) as usize;
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.pulse2_channel.timer_reload() & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;

                self.pulse2_channel.length_counter.set_counter(new_counter_load);
                self.pulse2_channel.set_timer_reload(new_timer);
                self.pulse2_channel.envelope.set_start_flag(true);
            }

            // Triangle Linear counter
            0x4008 => {
                let new_control = (data & 0x80) != 0;
                let new_reload = (data & 0x7F) as usize;

                self.triangle_channel.length_counter.set_halted(new_control);
                self.triangle_channel.linear_counter.set_control_flag(new_control);
                self.triangle_channel.linear_counter.set_reload_value(new_reload);
            }

            // Triangle Timer Low
            0x400A => {
                self.triangle_channel.set_timer_reload(
                    (self.triangle_channel.timer_reload & 0x700) | data as usize
                );
            }

            // Triangle Length counter & Timer High
            0x400B => {
                let new_timer_hi = (data & 0x7) as usize;
                let timer_lo = self.triangle_channel.timer_reload & 0xFF;
                let new_timer = (new_timer_hi << 8) | timer_lo;
                
                self.triangle_channel.set_timer_reload(new_timer);

                self.triangle_channel.length_counter.set_counter((data >> 3) as usize);
                self.triangle_channel.linear_counter.set_reload_flag(true);
            }

            // Noise Length Counter & Volume Envelope
            0x400C => {
                let new_halt = (data & 0x20) != 0;
                let new_const_volume = (data & 0x10) != 0;
                let new_volume = (data & 0x0F) as usize;

                // new_enable is also envelope_loop, idk if it needs to be flipped or no

                self.noise_channel.length_counter.set_halted(new_halt);
                self.noise_channel.envelope.set_loop_flag(new_halt);
                self.noise_channel.envelope.set_const_volume(new_const_volume);
                self.noise_channel.envelope.set_volume(new_volume);
            }

            // Noise Channel Mode & Period
            0x400E => {
                let new_mode = (data & 0x80) != 0;
                let new_period = data & 0x0F;

                self.noise_channel.mode = new_mode;
                self.noise_channel.set_period(new_period);
            }

            // Noise Channel Length counter
            0x400F => {
                let new_counter_load = (data >> 3) as usize;

                self.noise_channel.length_counter.set_counter(new_counter_load);
                self.noise_channel.envelope.set_start_flag(true);
            }

            0x4010 => {
                let new_irq_enable = (data & 0x80) != 0;
                let new_loop = (data & 0x40) != 0;
                let new_reload = (data & 0x0F) as usize;

                self.dmc_channel.set_irq_enable(new_irq_enable);
                self.dmc_channel.set_loop_flag(new_loop);
                self.dmc_channel.set_reload_value(new_reload);
            }

            // DMC PCM (Direct Access)
            0x4011 => {
                self.dmc_channel.set_output_direct(data & 0x7F);
            }

            // DMC Sample/Clip Address
            0x4012 => {
                let new_clip_address = 0xC000 + (64 * data as u16);

                self.dmc_channel.set_clip_address(new_clip_address)
            }

            // DMC Sample/Clip Length
            0x4013 => {
                let new_clip_length = (16 * data as usize) + 1;

                self.dmc_channel.set_clip_length(new_clip_length);
            }

            // Channel enable register
            0x4015 => {
                let pulse1_enabled = (data & 0x01) != 0;
                let pulse2_enabled = (data & 0x02) != 0;
                let triangle_enabled = (data & 0x04) != 0;
                let noise_enabled = (data & 0x08) != 0;
                let dmc_enabled = (data & 0x10) != 0;

                self.pulse1_channel.set_enable(pulse1_enabled);
                self.pulse2_channel.set_enable(pulse2_enabled);
                self.triangle_channel.set_enable(triangle_enabled);
                self.noise_channel.set_enable(noise_enabled);
                self.dmc_channel.set_enable(dmc_enabled);
            }

            // Frame update mode & frame interrupt register
            0x4017 => {
                let new_mode1 = data & 0x80 != 0;
                let new_irq_flag = data & 0x40 == 0;

                self.frame_update_mode1 = new_mode1;
                self.frame_update_counter = 0;
                if new_mode1 {
                    self.frame_update()
                }
                self.irq_request_flag = new_irq_flag;
            }

            _ => {}
        }
    }

    fn generate_sample(&mut self) -> f32 {
        let pulse1_sample = self.pulse1_channel.sample(self.clocks);
        let pulse2_sample = self.pulse2_channel.sample(self.clocks);
        let triangle_sample = self.triangle_channel.sample(self.clocks);
        let noise_sample = self.noise_channel.sample();
        let dmc_sample = self.dmc_channel.sample();

        let pulse_idx = (pulse1_sample + pulse2_sample) as usize;
        let tnd_idx = (3.0*triangle_sample + 2.0*noise_sample + dmc_sample) as usize;

        let pulse_out = Self::PULSE_LOOKUP[pulse_idx];
        let tnd_out = Self::TND_LOOKUP[tnd_idx];

        let sample = pulse_out + tnd_out;

        self.high_pass1.run(sample);
        self.high_pass2.run(sample);
        self.low_pass.run(sample);

        sample
    }

    fn push_sample(&mut self, sample: f32) {
        self.sample_batch.push(sample);

        if self.sample_batch.len() >= SAMPLE_BATCH_SIZE {
            self.sample_queue.lock().unwrap()
                .extend(self.sample_batch.drain(..));

            self.last_output = Instant::now();
            self.batches_sent += 1;
        }
    }

    pub fn audio_samples_queued(&self) -> usize {
        self.sample_queue.lock().unwrap().len()
    }

    fn frame_update(&mut self) {
        if self.frame_update_mode1 {
            match self.frame_update_counter {
                0 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                1 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                2 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                3 => {},
                4 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 5;
        } 
        else {
            match self.frame_update_counter {
                0 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                1 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();
                },
                2 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                },
                3 => {
                    self.update_envelopes();
                    self.clock_linear_counters();
                    self.update_length_counters();
                    self.update_sweepers();

                    self.trigger_irq = self.irq_request_flag;
                },
                _ => {},
            }

            self.frame_update_counter = (self.frame_update_counter + 1) % 4;
        }
    }

    fn clock_linear_counters(&mut self) {
        self.triangle_channel.update_linear_counter();
    }

    fn update_length_counters(&mut self) {
        self.pulse1_channel.update_length_counter();
        self.pulse2_channel.update_length_counter();
        self.triangle_channel.update_length_counter();
        self.noise_channel.update_length_counter();
    }

    fn update_sweepers(&mut self) {
        self.pulse1_channel.update_sweep();
        self.pulse2_channel.update_sweep();
    }

    fn update_envelopes(&mut self) {
        self.pulse1_channel.update_envelope();
        self.pulse2_channel.update_envelope();
        self.noise_channel.update_envelope();
    }

    pub fn trigger_irq(&self) -> bool {
        self.trigger_irq
    }

    pub fn dmc_trigger_irq(&self) -> bool {
        self.dmc_channel.irq_triggered()
    }

    pub fn set_trigger_irq(&mut self, val: bool) {
        self.trigger_irq = val;
    }

    pub fn set_dmc_irq_flag(&mut self, val: bool) {
        self.dmc_channel.set_irq_flag(val);
    }
}
