pub enum BatteryStatus {
    Dead,
    Low,
    OK,
    Full,
}

impl BatteryStatus {
    pub fn check_voltage() -> f32 {
        todo!();
    }

    pub fn check() -> BatteryStatus {
        let measuredvbat = Self::check_voltage();

        //DEBUG_PRINT(F("Battery: "));
        if measuredvbat < 3.3 {
          return Self::Dead;
        }
      
        if measuredvbat < 3.7 {
          //DEBUG_PRINTLN(F("LOW"));
          return Self::Low;
        }
      
        if measuredvbat < 4.1 {
          //DEBUG_PRINTLN(F("OK"));
          return Self::Low;
        }
      
        //DEBUG_PRINTLN(F("FULL"));
        Self::Full
    }
}
