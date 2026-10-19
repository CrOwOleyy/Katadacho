/** Vrai si `hour` (0 à 23) tombe dans la nuit, qui peut enjamber minuit (23 h à 7 h). */
export function isNight(hour: number, startHour: number, endHour: number): boolean {
  if (startHour === endHour) return false;
  return startHour < endHour ? hour >= startHour && hour < endHour : hour >= startHour || hour < endHour;
}
