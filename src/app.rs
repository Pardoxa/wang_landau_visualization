use egui::{
    emath::Align, Button, Color32, FontData, FontDefinitions, FontFamily, Layout, Vec2b, Visuals
};
use egui_plot::*;
use rand::SeedableRng;
use sampling::*;
use std::{time::{Instant, Duration}, thread};
use sampling::norm_log10_sum_to_1;
use crate::{CoinSeq, generate_cs};
use rand::distributions::Uniform;
use rand::distributions::Distribution;
pub struct SimData{
    c: CoinSeq
}

#[derive(PartialEq, Eq)]
pub enum Scale{
    Log,
    Lin
}
#[derive(PartialEq)]
pub enum LightMode{
    Light,
    Dark
}


pub struct AppState{
    sim: Option<SimData>,
    pause: bool,
    log_scale: bool,
    speed: f32,
    n: usize,
    log_f: Vec<[f64;2]>,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    pause_duration: Duration,
    log_f_logscale: bool,
    seed: u64,
    step_size: usize,
    pixel: f32,
    linewidth: f32,
    threshold: f64,
    refine_steps: usize,
    hist_scale: Scale,
    l_mode: LightMode,
    a_color: Color32,
    wl_color: Color32,
    e_color: Color32,
    s_color: Color32,
    show_simp_hist: bool,
    pairs: bool,
    f_steps: i32,
    noise: i32,
    best: bool,
    limit_to_1: bool,
    noise_seed: u64
}

impl Default for AppState{
    fn default() -> Self {
        Self { 
            sim: None, 
            pause: false, 
            log_scale: true, 
            speed: 1.2, 
            n: 1500, 
            log_f: Vec::new(), 
            start_time: None, 
            log_f_logscale: false,
            step_size: 1,
            seed: 834628956578,
            pixel: 2.0,
            linewidth: 1.5,
            threshold: 0.000001,
            pause_time: None,
            pause_duration: Duration::new(0, 0),
            refine_steps: 30000000,
            hist_scale: Scale::Lin,
            l_mode: LightMode::Light,
            a_color: Color32::from_rgb(0x_D8, 0x_1B, 0x_60),
            s_color: Color32::BLACK,
            wl_color: Color32::from_rgb(0x_1E, 0x_88, 0x_E5),
            e_color: Color32::from_rgb(0x_ff, 0x_C1, 0x_07),
            show_simp_hist: false,
            pairs: false,
            f_steps: 0,
            noise: 0,
            best: false,
            limit_to_1: false,
            noise_seed: 1238947
        }
    }
}

impl AppState{
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}
        cc.egui_ctx.set_visuals(Visuals::light());
        cc.egui_ctx.set_pixels_per_point(2.0);
        

        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "my_font".to_owned(),
            FontData::from_static(include_bytes!("../DejaVu_Sans/DejaVuSans.ttf"))); // .ttf and .otf supported
        fonts.families.get_mut(&FontFamily::Proportional).unwrap()
            .insert(0, "my_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);
        
        Default::default()

    }
}


impl eframe::App for AppState {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);

        // DO NOT SAVE
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            pause,
            sim,
            log_scale,
            speed,
            n,
            log_f,
            start_time,
            log_f_logscale,
            seed,
            step_size,
            pixel,
            linewidth,
            threshold,
            pause_duration,
            pause_time,
            refine_steps,
            hist_scale,
            l_mode,
            a_color,
            s_color,
            e_color,
            wl_color,
            show_simp_hist,
            pairs,
            f_steps,
            noise,
            best,
            limit_to_1,
            noise_seed
        } = self;
        //// Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui


        egui::SidePanel::left("side_panel")
            .default_width(300.0)
            .show(ctx, |ui| {


            egui::ScrollArea::both().show(
                ui,
                |ui|
                {

                    match l_mode{
                        LightMode::Dark => {
                            if ui.add(
                                 Button::new("☀").frame(false)
                             ).on_hover_text("Wechsel in den hellen Modus")
                             .clicked(){
                                ui.ctx().set_visuals(Visuals::light());
                                *l_mode = LightMode::Light;
                            }
                        },
                        LightMode::Light => {
                            if ui.add(
                                 Button::new("🌙").frame(false)
                             ).on_hover_text("Wechsel in den Dunklen Modus")
                             .clicked(){
                                ui.ctx().set_visuals(Visuals::dark());
                                *l_mode = LightMode::Dark;
                            }
                        }
                    }

                    if ui.add(egui::Button::new("Start"))
                        .on_hover_text("Startet die Simulation.")
                        .clicked()
                    {
                        *sim = Some(
                            SimData { c: generate_cs(*n, *seed, *step_size, *threshold) }
                        );
                        *log_f = Vec::new();
                        *start_time = Some(Instant::now());
                        *pause_duration = Duration::new(0, 0);
                        if let Some(ins) = pause_time
                        {
                            *ins = Instant::now();
                        }
                    }
                    let btn_text = if *pause{
                        "Fortfahren"
                    } else{
                        "Pausieren"
                    };
                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *pause = !*pause;
                        if *pause {
                            *pause_time = Some(Instant::now());
                        } else {
                            let dur = pause_time.as_ref().unwrap().elapsed();
                            *pause_time = None;
                            *pause_duration += dur;
                        }
                    }

                    let btn_text = if *log_scale {
                        "Switch to linear Scale"
                    } else {
                        "Switch to logscale"
                    };

                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *log_scale = !*log_scale;
                    }
                    ui.add(egui::Slider::new(speed, 0.0..=10.0).logarithmic(false).text("Speed"));
                    ui.add(egui::Slider::new(n, 10..=10000).logarithmic(true).text("N"));
                    ui.add(egui::Slider::new(seed, 0..=u64::MAX).logarithmic(true).text("Seed"));
                    ui.add(egui::Slider::new(step_size, 1..=30).logarithmic(false).text("step size"));

                    let btn_text = if *log_f_logscale {
                        "to_log_f"
                    } else {
                        "to log_log_f"
                    };

                    if ui.add(egui::Button::new(btn_text))
                        .clicked()
                    {
                        *log_f_logscale = !*log_f_logscale;
                    }

                    ui.add(egui::Slider::new(pixel, 1.0..=5.0).logarithmic(false).text("Zoom"));
                    if ui.add(egui::Button::new("Rescale"))
                        .clicked()
                    {
                        ctx.set_pixels_per_point(*pixel);
                    }

                    ui.add(egui::Slider::new(linewidth, 0.0..=10.0).logarithmic(false).text("line"));
                    ui.add(egui::Slider::new(threshold, 0.00000000001..=0.001).logarithmic(true).text("threshold"));
                    ui.add(egui::Slider::new(refine_steps, 100000..=10000000000).logarithmic(true).text("E refine"));
                    ui.radio_value(hist_scale, Scale::Lin, "Hist Lin");
                    ui.radio_value(hist_scale, Scale::Log, "Hist Log");

                    ui.color_edit_button_srgba(a_color);
                    ui.color_edit_button_srgba(s_color);
                    ui.color_edit_button_srgba(e_color);
                    ui.color_edit_button_srgba(wl_color);

                    ui.checkbox(show_simp_hist, "Simp Hist");

                    let text = if *pairs {
                        "Normal"
                    } else {
                        "Pairs"
                    };

                    if ui.add(egui::Button::new(text))
                        .clicked()
                    {
                        *pairs = !*pairs;
                    }
                    ui.checkbox(best, "Noise");
                    if *best{
                        ui.add(egui::Slider::new(f_steps, 0..=7).logarithmic(false).text("Best PR"));
                        ui.add(egui::Slider::new(noise, 0..=30).logarithmic(false).text("noise"));
                        ui.add(egui::Slider::new(noise_seed, 0..=2000012).logarithmic(false).text("noise seed"));
                        ui.checkbox(limit_to_1, "limit to 1");
                    }
                }
            );
            
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            //ui.heading("Random Walker");
            //ui.hyperlink("https://github.com/emilk/eframe_template");
            //ui.add(egui::github_link_file!(
            //    "https://github.com/emilk/eframe_template/blob/master/",
            //    "Source code."
            //));

            if let Some(sim_data) = sim{

                let time = Instant::now();
                if !*pause
                {
                    let s = *speed;
                    let wl = sim_data.c.wl.clone();
                    let t = thread::spawn(
                        move || {
                            wl.write().unwrap().wang_landau_while_acc(
                                |ensemble, step, old_energy| {
                                    ensemble.update_head_count(step, old_energy)
                                }, 
                                |_| {(time.elapsed().as_millis() as f32) < (30.0_f32 * s)}
                            );
                        }
                    );

                    let simp = sim_data.c.simple.clone();

                    let t2 = thread::spawn(
                        move ||
                        {
                            simp.lock().sample_while(|| (time.elapsed().as_millis() as f32) < (30.0_f32 * s))
                        }
                    );

                    sim_data.c.entr.entropic_sampling_while_acc(
                        |ensemble, step, old_energy| {
                            ensemble.update_head_count(step, old_energy)
                        }, 
                        |_| {}, 
                        |_| {(time.elapsed().as_millis() as f32) < (30.0_f32 * s)}
                    );

                    if sim_data.c.entr.step_counter() > *refine_steps{
                        sim_data.c.entr.refine_estimate();
                    }

                    t.join().unwrap();
                    t2.join().unwrap();
                }

                if !*pause && ! sim_data.c.wl.read().unwrap().is_finished() {
                    let current_log_f: f64 = sim_data.c.wl.read().unwrap().log_f();
                    let ellased = start_time.as_ref().unwrap().elapsed() - *pause_duration;
                    log_f.push([ellased.as_secs_f64(), current_log_f]);
                }
                

                let layout = Layout{
                    main_dir: egui::Direction::LeftToRight,
                    main_wrap: false,
                    main_align: Align::Center,
                    main_justify: true,
                    cross_align: Align::Min,
                    cross_justify: true
                };

               ui.with_layout(
                        layout, 
                        |ui|{
                            let max_width = ui.available_width();
                            let mut density = sim_data.c.wl.read().unwrap().log_density_base10();
                            let len = density.len();
                            let mut true_density = sim_data.c.log_prob_true.clone();
                            
                            let mut e_data: Vec<_> = sim_data.c.entr.log_density_estimate()
                                .iter().map(|val|  *val * std::f64::consts::LOG10_E)
                                .collect();
                            norm_log10_sum_to_1(&mut e_data);
                            norm_log10_sum_to_1(&mut density);
                            norm_log10_sum_to_1(&mut true_density);

                            let total = 2.0_f64.powi(-*f_steps);

                            let num = 2_u64.pow(*f_steps as u32) * 8;

                            

                            let list: Vec<_> = (0..=num)
                                .map(
                                    |v|
                                    {
                                        let v = v as f64 * total;
                                        (-v).exp()
                                    }
                                ).collect();

                            let mut rng = rand_pcg::Pcg64::seed_from_u64(*noise_seed);
                            let closes = |val: f64| {
                                let r = list[0];
                                let mut diff = (val-r).abs();
                                let mut idx = 0;
                                for (v, i) in list[1..].iter().zip(1..)
                                {
                                    let n_diff = (val-*v).abs();
                                    if n_diff < diff {
                                        diff = n_diff;
                                        idx = i;
                                    }
                                }
                                idx
                            };

                            let mut best_estimate: Vec<_> = true_density.windows(2)
                                    .map(
                                        |arr| 
                                        closes( 10_f64.powf(-(arr[0] - arr[1]).abs()))
                                    )
                                    .collect();

                            let uni = Uniform::new(0.0, 1.0);
                            let other = Uniform::new_inclusive(0, *noise);
                            for i in 1..(best_estimate.len()-1)
                            {
                                let p = (0.5 - i as f64 / (best_estimate.len()-1) as f64).abs() * 2.0 + 0.075;
                                if uni.sample(&mut rng) < (p*p)  {
                                    let v = other.sample(&mut rng);
                                    best_estimate[i-1] += v;
                                    best_estimate[i] -= v;
                                }
                                
                                
                            }

                            fn with_limit(v: f64) -> f64
                            {
                                v.min(0.0).exp()
                            }

                            fn without_limit(v: f64) -> f64
                            {
                                v.exp()
                            }

                            let fun = if *limit_to_1{
                                with_limit
                            } else {
                                without_limit
                            };

                            let mut best_estimate: Vec<[f64;2]> = best_estimate
                            .iter()
                            .enumerate()
                            .map(
                                |(i, arr)| 
                                [i as f64 / len as f64, 
                                    fun(-*arr as f64 * total)
                                ] 
                            )
                            .collect();

                            if !*pairs{
                                let start = -10.0;
                                let mut other = vec![start];

                                best_estimate.iter()
                                    .for_each(
                                        |v|
                                        {
                                            let last = other.last().unwrap();
                                            
                                            if v[0] < 0.5 {
                                                let r = -v[1].log10()+last;
                                                other.push(r);
                                            } else {
                                                let r = v[1].log10()+last;
                                                other.push(r);
                                            }
                                        }
                                    );
                                norm_log10_sum_to_1(&mut other);
                                best_estimate = 
                                    other.iter()
                                    .enumerate()
                                    .map(|(i, v)| [i as f64 / len as f64, *v] )
                                .collect();
                                
                                if !*log_scale {
                                    best_estimate.iter_mut()
                                        .for_each(|v| v[1] = 10_f64.powf(v[1]));
                                }
                                    
                            } 

                            let simp_data = if *log_scale{
                                sim_data.c.simple.lock().get_prob_log10()
                            } else {
                                sim_data.c.simple.lock().get_prob()
                            };
                            if !*log_scale
                            {
                                density.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                                true_density.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                                e_data.iter_mut()
                                    .for_each(|val| *val = 10.0f64.powf(*val));
                            }
                            

                            let density: Vec<_> = if *pairs {
                                density.windows(2)
                                    .map(|arr| 10_f64.powf(-(arr[0] - arr[1]).abs()) )
                                    .collect()  
                            } else {
                                density
                            };

                            let wl_density: Vec<_> = 
                                density.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();
    
                            let true_density: Vec<_> = if *pairs {
                                true_density.windows(2)
                                    .map(|arr| 10_f64.powf(-(arr[0] - arr[1]).abs()) )
                                    .collect()  
                            } else {
                                true_density
                            };



                            let true_density: Vec<_> = 
                                true_density.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();

                            let e_data: Vec<_> = if *pairs {
                                e_data.windows(2)
                                    .map(|arr| 10_f64.powf(-(arr[0] - arr[1]).abs()) )
                                    .collect()  
                            } else {
                                e_data
                            };

                            let e_density: Vec<_> = 
                                e_data.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();

                            let simp_data: Vec<_> = if *pairs {
                                simp_data.windows(2)
                                    .map(|arr| 10_f64.powf(-(arr[0] - arr[1]).abs()) )
                                    .collect()  
                            } else {
                                simp_data
                            };

                            let s_density: Vec<_> = 
                                simp_data.into_iter()
                                    .enumerate()
                                .map(
                                    |(idx, den)|
                                    {
                                        let x = idx as f64 / len as f64;
                                        let y = den;
                                        [x,y]
                                    }
                                ).collect();


                            ui.vertical(
                                |ui|
                                {
                                    let legend = Legend::default().position(Corner::RightBottom)
                                        .background_alpha(0.5);
                                    let hight = ui.available_height();
                                    let mut p = Plot::new("plot_average_etc")
                                    .include_x(0.0)
                                    .x_axis_formatter(|g, _, _| format!("{}", g.value));

                                    if *log_scale && !*pairs{
                                        p = p.y_axis_formatter(
                                            |g, _,_| 
                                            {
                                                let s = format!("{}", g.value);
                                                let ex: String = s.chars().map(exchange).collect();
                                                format!("10{ex}")
                                            }
                                        );
                                    }
                                    
                                    p.x_grid_spacer(
                                        |_|
                                        {
                                            vec![
                                                GridMark { value: 0.0, step_size: 0.5 },
                                                GridMark { value: 0.25, step_size: 0.5 },
                                                GridMark { value: 0.5, step_size: 0.5 },
                                                GridMark { value: 0.75, step_size: 0.5 },
                                                GridMark { value: 1.0, step_size: 0.5 },
                                            ]
                                        }
                                    )
                                    .legend(legend)
                                    .height(hight - 25.0)
                                    .width(max_width * 0.5)
                                    .y_axis_label("Probability of heads rate")
                                    .x_axis_label("Heads rate")
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            
                                            let true_line = Line::new(true_density).name("analytic Results")
                                                .width(*linewidth*2.0)
                                                .color(*a_color);

                                            plot_ui.line(true_line);
                                            if *pairs {
                                                let wl_points = Points::new(wl_density)
                                                .name("WL Results")
                                                .radius(*linewidth*0.8)
                                                .color(*wl_color);
                                                plot_ui.points(wl_points);



                                            } else {
                                                let wl_line = Line::new(wl_density).name("WL Results")
                                                .width(*linewidth)
                                                .color(*wl_color);
                                                plot_ui.line(wl_line);
                                            }
                                            if *best{
                                                if *log_scale{
                                                    let p = Points::new(best_estimate)
                                                    .name("best")
                                                    .radius(*linewidth*0.7)
                                                        .color(Color32::DARK_GRAY);
                                                    plot_ui.points(p);
                                                }else {
                                                    let p = Line::new(best_estimate)
                                                    .name("best")
                                                    .width(*linewidth*0.9)
                                                        .color(Color32::DARK_GRAY);
                                                    plot_ui.line(p);
                                                }
                                                
                                            }

                                            
                                            
                                            let ent_line = Line::new(e_density).name("Entropic Results")
                                                .width(*linewidth)
                                                .color(*e_color);
                                            let s_points = Points::new(s_density)
                                                .name("Simple Results")
                                                .radius(*linewidth*0.9)
                                                .shape(MarkerShape::Cross)
                                                .color(*s_color);
                                            
                                            
                                            
                                            plot_ui.line(ent_line);
                                            plot_ui.points(s_points);
                                            
                                            
                                        }
                                    );
                                }
                            );
                            ui.vertical(
                                |ui|
                                {
                                    let name = if *log_f_logscale{
                                        "log10(logE(f))"
                                    } else {
                                        "logE(f)"
                                    };

                                    

                                    let hight = ui.available_height();
                                    Plot::new("plot_log_f")
                                    .include_x(0.0)
                                    .include_y(0.0)
                                    .auto_bounds(Vec2b::new(true, true))
                                    .legend(Legend::default())
                                    .height((hight - 25.0)*0.5)
                                    .y_axis_label(name)
                                    .x_axis_label("Run time in seconds")
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            let mut tmp_log_f = log_f.clone();
                                            if *log_f_logscale
                                            {
                                                tmp_log_f.iter_mut()
                                                    .for_each(|[_, val]| *val = val.log10());
                                            }
                                            
                                            let log_f_line = Line::new(tmp_log_f).name(name)
                                                .width(*linewidth)
                                                .color(*wl_color);
                                            
    
                                            plot_ui.line(log_f_line);
                                            
                                        }
                                    );
                                    let mut hist: Vec<_> = sim_data.c.wl.read().unwrap().hist().bin_hits_iter()
                                        .map(|(bin, hits)| [bin as f64 / len as f64, hits as f64])
                                        .collect();
                                    let mut ent_hist: Vec<_> = sim_data.c.entr.hist().bin_hits_iter()
                                        .map(|(bin, hits)| [bin as f64 / len as f64, hits as f64])
                                        .collect();

                                    let mut s_hist: Vec<_> = sim_data.c.simple.lock().hist.bin_hits_iter()
                                        .map(|(bin, hits)| [bin as f64 / len as f64, hits as f64])
                                        .collect();

                                    if matches!(*hist_scale, Scale::Log) {
                                        hist.iter_mut()
                                            .for_each(
                                                |[_, val]|
                                                {
                                                    if *val < 1.0 {
                                                        *val = f64::NAN;   
                                                    } else {
                                                        *val = val.log10();
                                                    }
                                                    
                                                }
                                            );
                                        ent_hist.iter_mut()
                                            .for_each(
                                                |[_, val]|
                                                {
                                                    if *val < 1.0 {
                                                        *val = f64::NAN;   
                                                    } else {
                                                        *val = val.log10();
                                                    }
                                                }
                                            );
                                        s_hist.iter_mut()
                                            .for_each(
                                                |[_, val]|
                                                {
                                                    if *val < 1.0 {
                                                        *val = f64::NAN;   
                                                    } else {
                                                        *val = val.log10();
                                                    }
                                                }
                                            );
                                        
                                    }

                                    let hight = ui.available_height();
                                    Plot::new("plot_histogram")
                                    .include_x(0.0)
                                    .include_y(0.0)
                                    .auto_bounds(Vec2b::new(false, true))
                                    .legend(Legend::default())
                                    .height(hight - 25.0)
                                    .x_axis_label("histogram")
                                    .y_axis_label("#hits")
                                    .show(
                                        ui, 
                                        |plot_ui|
                                        {
                                            
                                            let histogram = Line::new(hist).name("Wang Landau Histogram")
                                                .width(*linewidth)
                                                .color(*wl_color);
                                            
                                            let ent_line = Line::new(ent_hist).name("Entropic Histogram")
                                                .width(*linewidth)
                                                .color(*e_color);

                                            
    
                                            plot_ui.line(histogram);
                                            plot_ui.line(ent_line);

                                            if *show_simp_hist{
                                                let s_line = Line::new(s_hist).name("Simple Histogram")
                                                    .width(*linewidth)
                                                    .color(*s_color);
                                                plot_ui.line(s_line);
                                            }
                                            
                                        }
                                    );
                                    ctx.request_repaint();
                                }
                            );
                        }
                    );
            }
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
        
    }
}


fn exchange(c: char) -> char
{
    
    let super_list = [
        '⁰',
        '¹',
        '²',
        '³',
        '⁴',
        '⁵',
        '⁶',
        '⁷',
        '⁸',
        '⁹',
        '⁻'
    ];
    let old_list = [
        '0',
        '1',
        '2',
        '3',
        '4',
        '5',
        '6',
        '7',
        '8',
        '9',
        '-'
    ];
    if let Some(p) = old_list.iter().position(|&x| x == c){
        super_list[p]
    } else {
        c
    }
    
}