import java.lang.reflect.Method;
import java.util.Arrays;

public final class MotaaNumericOracle {
    public static void main(String[] args) throws Exception {
        if (args.length == 1 && args[0].equals("ayanamsha")) {
            Class<?> swiss = Class.forName("L0.D");
            Object instance = swiss.getConstructor().newInstance();
            Method select = swiss.getMethod("u", int.class);
            Method mean = swiss.getMethod("r", double.class);
            for (double tt = 2378496.5; tt < 2488434.5 + 3652.5; tt += 3652.5) {
                double[] entry = new double[10];
                entry[0] = Math.min(tt, 2488434.5);
                for (int mode = 0; mode < 9; mode++) {
                    select.invoke(instance, mode == 0 ? 1 : mode == 1 ? 0 : mode);
                    entry[mode + 1] = (double) mean.invoke(instance, entry[0]);
                }
                System.out.println("ayanamsha=" + Arrays.toString(entry));
            }
            return;
        }
        if (args.length == 1 && args[0].equals("delta-t")) {
            Class<?> time = Class.forName("L0.w");
            Method julian = time.getMethod("f", int.class, int.class, int.class, double.class, boolean.class);
            Method delta = time.getMethod("b", double.class);
            for (int year = 1800; year <= 2101; year++) {
                for (int month = 1; month <= 12; month++) {
                    for (int day : new int[]{1, 16}) {
                        double jdut = (double) julian.invoke(null, year, month, day, 0.0, true);
                        double seconds = (double) delta.invoke(null, jdut) * 86400.0;
                        System.out.println("delta=[" + jdut + "," + seconds + "]");
                    }
                    if (year == 2101) return;
                }
            }
            return;
        }
        if (args.length == 1 && args[0].equals("myanmar")) {
            Method watat = Class.forName("L0.c").getMethod("a", double.class);
            int[] watatYears = {
                    205, 246, 471, 572, 651, 653, 656, 672, 729, 767, 797, 798, 813,
                    1099, 1100, 1201, 1202, 1216, 1217, 1234, 1261, 1263, 1264,
                    1311, 1312, 1344, 1345, 1377, 1385, 1386, 1388
            };
            for (int year : watatYears) {
                @SuppressWarnings("unchecked")
                java.util.HashMap<String, Double> result =
                        (java.util.HashMap<String, Double>) watat.invoke(null, (double) year);
                long fm = Math.round(result.get("fm"));
                long isWatat = Math.round(result.get("watat"));
                System.out.println("myanmar_watat=[" + year + "," + fm + "," + isWatat + "]");
            }
            Method a0 = Class.forName("motaa.myanmar.main")
                    .getMethod("A0", int.class, int.class, int.class);
            int[][] dates = {
                    {1984, 1, 3}, {2000, 1, 1}, {2019, 4, 17}, {2024, 1, 1},
                    {2024, 4, 17}, {2025, 4, 13}, {2025, 4, 18}, {2026, 10, 22},
                    {1980, 1, 1}, {1752, 9, 2}, {1752, 9, 14}, {1752, 9, 15},
                    {2024, 2, 29}, {2023, 7, 18}, {2023, 8, 1}, {2023, 8, 30},
                    {2026, 7, 29}, {2026, 8, 28}, {1962, 3, 2}, {1885, 7, 14}
            };
            for (int[] date : dates) {
                String text = (String) a0.invoke(null, date[2], date[1], date[0]);
                String escaped = text.replace("\\", "\\\\").replace("\"", "\\\"");
                System.out.println("myanmar_a0=[" + date[0] + "," + date[1] + "," + date[2] + ",\"" + escaped + "\"]");
            }
            return;
        }
        if (args.length == 1 && args[0].equals("calendar")) {
            Method midnight = Class.forName("motaa.myanmar.main")
                    .getMethod("w0", int.class, int.class, int.class);
            for (int[] date : new int[][]{
                    {-4712, 1, 1}, {-100, 3, 1}, {-1, 12, 31}, {0, 1, 1},
                    {1500, 2, 28}, {1582, 10, 4}, {1582, 10, 5}, {1582, 10, 14},
                    {1582, 10, 15}, {1600, 2, 29}, {1700, 3, 1}, {1900, 3, 1},
                    {2000, 2, 29}, {2100, 3, 1}, {9999, 12, 31},
            }) {
                double value = (double) midnight.invoke(null, date[0], date[1], date[2]);
                System.out.println("calendar=[" + date[0] + "," + date[1] + "," + date[2] + "," + value + "]");
            }
            return;
        }
        if (args.length == 1 && args[0].equals("rahu")) {
            Class<?> renderer = Class.forName("motaa.myanmar.main");
            String[] methods = {"B0", "L0", "M0", "X0", "Y0"};
            Method[] classifiers = new Method[methods.length];
            for (int index = 0; index < methods.length; index++) {
                classifiers[index] = renderer.getMethod(methods[index], double.class);
            }
            for (int index = 0; index < 27; index++) {
                double boundary = index * (360.0 / 27.0);
                for (double longitude : new double[]{boundary, boundary + 1.0, boundary + 1e-8}) {
                    Object[] outputs = new Object[classifiers.length];
                    for (int method = 0; method < classifiers.length; method++) {
                        outputs[method] = classifiers[method].invoke(null, longitude);
                    }
                    System.out.println("rahu=[" + longitude + "," + Arrays.deepToString(outputs) + "]");
                }
            }
            return;
        }
        Class<?> engine = Class.forName("L0.z");
        Method planet = engine.getMethod("l", int.class, int.class, int.class, double.class, double.class, int.class, String.class, String.class);
        Method declination = engine.getMethod("b", int.class, int.class, int.class, double.class, double.class, int.class, String.class, String.class);
        Method speed = engine.getMethod("i", int.class, int.class, int.class, double.class, double.class, int.class, String.class, String.class);
        Class<?>[] houseArgs = {int.class, int.class, int.class, double.class, double.class, double.class, double.class, String.class, String.class, String.class};
        Method cusps = engine.getMethod("g", houseArgs);
        Method midpoints = engine.getMethod("m", houseArgs);
        Method angles = engine.getMethod("c", houseArgs);
        int year = Integer.parseInt(args[0]);
        int month = Integer.parseInt(args[1]);
        int day = Integer.parseInt(args[2]);
        double localTime = Double.parseDouble(args[3]);
        double eastOffset = Double.parseDouble(args[4]);
        double latitude = Double.parseDouble(args[5]);
        double longitude = Double.parseDouble(args[6]);
        String zodiac = args[7], mode = args[8];
        Class<?> time = Class.forName("L0.w");
        Method calendar = engine.getMethod("j", int.class, int.class, int.class);
        Method julian = time.getMethod("f", int.class, int.class, int.class, double.class, boolean.class);
        boolean gregorian = (boolean) calendar.invoke(null, year, month, day);
        double jdut = (double) julian.invoke(null, year, month, day, localTime, gregorian) - eastOffset / 24.0;
        double deltaT = (double) time.getMethod("b", double.class).invoke(null, jdut) * 86400.0;
        double tropicalMoon = (double) planet.invoke(null, day, month, year, localTime, -eastOffset, 1, "Tropical", mode);
        System.out.println("moon_clock=" + Arrays.toString(new double[]{jdut, deltaT, tropicalMoon}));
        Class<?> swiss = Class.forName("L0.D");
        Object lunarEngine = swiss.getConstructor().newInstance();
        Method calc = swiss.getMethod("a", double.class, int.class, int.class, double[].class, StringBuffer.class);
        double[] moonState = new double[6];
        StringBuffer moonError = new StringBuffer();
        // Geocentric rectangular position and velocity, true position, mean ecliptic.
        int resultFlags = (int) calc.invoke(lunarEngine, jdut + deltaT / 86400.0, 1,
                4 | 16 | 64 | 256 | 4096, moonState, moonError);
        if (resultFlags < 0) throw new IllegalStateException(moonError.toString());
        System.out.println("moon_state=" + Arrays.toString(moonState));
        int[] swissIds = {0, 1, 4, 2, 5, 3, 6, 11, 10, 7, 8, 9};
        double[] longitudes = new double[swissIds.length];
        double[] tropicalLongitudes = new double[swissIds.length];
        double[] declinations = new double[swissIds.length];
        double[] distances = new double[swissIds.length];
        double[] speeds = new double[swissIds.length];
        for (int i = 0; i < swissIds.length; i++) {
            longitudes[i] = (double) planet.invoke(null, day, month, year, localTime, -eastOffset, swissIds[i], zodiac, mode);
            tropicalLongitudes[i] = zodiac.equals("Tropical") ? longitudes[i]
                    : (double) planet.invoke(null, day, month, year, localTime, -eastOffset, swissIds[i], "Tropical", mode);
            declinations[i] = (double) declination.invoke(null, day, month, year, localTime, -eastOffset, swissIds[i], zodiac, mode);
            double[] coordinates = (double[]) speed.invoke(null, day, month, year, localTime, -eastOffset, swissIds[i], zodiac, mode);
            speeds[i] = coordinates[3];
            distances[i] = coordinates[2];
        }
        Class<?> renderer = Class.forName("motaa.myanmar.main");
        int[] dasha = (int[]) renderer.getMethod("v", double.class).invoke(null, longitudes[1]);
        double mansion = longitudes[1] / (360.0 / 27.0);
        double balance = dasha[2] * 365.2422 * (1.0 - (mansion - (int) mansion));
        System.out.println("dasha=[" + dasha[1] + "," + balance + "]");
        System.out.println("declinations=" + Arrays.toString(declinations));
        System.out.println("distances=" + Arrays.toString(distances));
        System.out.println("speeds=" + Arrays.toString(speeds));
        System.out.println("positions=" + Arrays.toString(longitudes));
        System.out.println("tropical_positions=" + Arrays.toString(tropicalLongitudes));
        for (String system : new String[]{"Equal", "Vehlow_Placidus", "Placidus", "Koch", "Porphyrius", "Regiomontanus", "Campanus", "Vehlow_Equal"}) {
            Object[] input = {day, month, year, localTime, -eastOffset, latitude, longitude, zodiac, mode, system};
            System.out.println(system + "=" + Arrays.toString((double[]) cusps.invoke(null, input)));
            double[] chartAngles = (double[]) angles.invoke(null, input);
            Class<?>[] signature = new Class<?>[13];
            Arrays.fill(signature, double.class);
            Object[] aspectInputs = {
                chartAngles[0], longitudes[0], longitudes[1], longitudes[2], longitudes[3],
                longitudes[4], longitudes[5], longitudes[6], longitudes[7], longitudes[9],
                longitudes[10], longitudes[11], chartAngles[1],
            };
            if (system.equals("Equal")) {
                System.out.println("angles=" + Arrays.toString(chartAngles));
                Method h = renderer.getMethod("H", signature);
                System.out.println("western=" + Arrays.deepToString((byte[][]) h.invoke(null, aspectInputs)));
            }
            Method g = renderer.getMethod("G", signature);
            double[] houseMidpoints = (double[]) midpoints.invoke(null, input);
            System.out.println("centers_" + system + "=" + Arrays.toString(Arrays.copyOfRange(houseMidpoints, 1, 13)));
            byte[][] houseCodes = new byte[12][];
            for (int house = 0; house < 12; house++) {
                aspectInputs[0] = houseMidpoints[house + 1];
                houseCodes[house] = (byte[]) g.invoke(null, aspectInputs);
            }
            System.out.println("western_" + system + "=" + Arrays.deepToString(houseCodes));
        }
    }
}
