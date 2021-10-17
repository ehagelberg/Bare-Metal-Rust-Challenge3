#![no_std]
#![no_main]

use panic_halt as _;
use embedded_graphics::mono_font::{
    ascii::FONT_10X20,
    MonoTextStyleBuilder,
};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use embedded_graphics::text::Text;
use longan_nano::{lcd, lcd_pins};
use longan_nano::hal::{eclic::{EclicExt, Level, LevelPriorityBits, Priority, TriggerType}, pac, prelude::*};
use longan_nano::hal::timer::{Event, Timer};
use riscv_rt::entry;
use heapless::String;

static mut NUMBER: i32 = 0;
static mut TIMER: Option<Timer<longan_nano::hal::pac::TIMER1>> = None;


#[allow(non_snake_case)]
#[no_mangle]
fn TIMER1(){

    unsafe{

        riscv::interrupt::disable();
        NUMBER += 1;
        TIMER.as_mut().unwrap().clear_update_interrupt_flag();
        riscv::interrupt::enable();
        
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);

    //lcd config
    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    
    //set timer
    unsafe{
        let mut timer = Timer::timer1(dp.TIMER1, 1.hz(), &mut rcu);
        timer.listen(Event::Update);
        TIMER = Some(timer);
    }

    //ECLIC setup
    longan_nano::hal::pac::ECLIC::reset();
    longan_nano::hal::pac::ECLIC::set_level_priority_bits(LevelPriorityBits::L0P4);
    longan_nano::hal::pac::ECLIC::set_threshold_level(Level::L1);
    longan_nano::hal::pac::ECLIC::setup(longan_nano::hal::pac::Interrupt::TIMER1, TriggerType::Level, Level::L1, Priority::P1);
    unsafe{
        longan_nano::hal::pac::ECLIC::unmask(longan_nano::hal::pac::Interrupt::TIMER1)
    };

    //enable interrupts
    unsafe{riscv::interrupt::enable()};


    //set style and white rectangle to the screen
    let style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::RED)
        .background_color(Rgb565::WHITE)
        .build();
    Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(&mut lcd)
        .unwrap();

    loop{       
        
        unsafe{
            //set text from counter
            let s = String::<32>::from(NUMBER);
            Text::new(&s, Point::new(45, 45), style)
            .draw(&mut lcd)
            .unwrap();
        }
        
        //set chip to sleep
        unsafe{riscv::asm::wfi();}
    }
}

