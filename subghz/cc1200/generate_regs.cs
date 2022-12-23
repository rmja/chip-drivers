// This is a C# tool to generate the register bitfields from the register definitions included in SmartRF studio.

using System.Globalization;
using System.Text.RegularExpressions;
using System.Xml;
using System.Xml.Serialization;

var serializer = new XmlSerializer(typeof(RegisterDefinition), new[]
{
    typeof(Register), typeof(Bitfield), typeof(Value)
});

await using var file = File.OpenRead("C:\\Program Files (x86)\\Texas Instruments\\SmartRF Tools\\SmartRF Studio 7\\config\\xml\\cc1200\\register_definition.xml");
using var reader = XmlReader.Create(file, new XmlReaderSettings
{
    DtdProcessing = DtdProcessing.Ignore
});
var definition = (RegisterDefinition)serializer.Deserialize(reader)!;
var trimRegex = new Regex("<TABLE.*</TABLE>", RegexOptions.Singleline | RegexOptions.Compiled);

foreach (var register in definition.Register)
{
    var name = GetStructName(register.Name);
    Console.WriteLine($"bitfield! {{");
    Console.WriteLine($"    /// {register.Description}");
    Console.WriteLine($"    ///");
    Console.WriteLine($"    /// # Address");
    Console.WriteLine($"    ///");
    Console.WriteLine($"    /// The address of this register is {register.Address}");
    Console.WriteLine($"    #[derive(Clone, Copy)]");
    Console.WriteLine($"    pub struct {name}(u8);");
    foreach (var bitfield in register.Bitfield)
    {
        var getter = bitfield.Name.ToLowerInvariant();
        var setter = bitfield.Access == "R/W"
            ? "set_" + getter
            : "_";
        var size = bitfield.Start - bitfield.Stop + 1;
        var range = size == 1
            ? bitfield.Start.ToString()
            : $"{bitfield.Start}, {bitfield.Stop}";

        Console.WriteLine();
        if (!string.IsNullOrEmpty(bitfield.Description))
        {
            var description = bitfield.Description.ReplaceLineEndings().Trim();
            description = trimRegex.Replace(description, "");

            using var lineReader = new StringReader(description);
            string? line;
            while ((line = lineReader.ReadLine()) != null)
            {
                Console.WriteLine($"    /// {line}");
            }
        }

        if (bitfield.Value.Any())
        {
            Console.WriteLine($"    ///");
            Console.WriteLine($"    /// # Values");
            Console.WriteLine($"    ///");

            foreach (var value in bitfield.Value)
            {
                Console.WriteLine($"    /// - {value.Number}b: {value.Brief}");
            }

            Console.WriteLine($"    ///");
            Console.WriteLine($"    /// The default value is {bitfield.Reset}");
        }
        Console.WriteLine($"    pub {getter}, {setter}: {range};");
    }
    Console.WriteLine($"}}");
    Console.WriteLine();

    var defaultNumber = int.Parse(register.RegReset.Replace("0x", ""), NumberStyles.HexNumber);
    var defaultString = "0x" + Convert.ToString(defaultNumber, 16).PadLeft(2, '0');
    Console.WriteLine($"impl Default for {name} {{");
    Console.WriteLine($"    fn default() -> Self {{");
    Console.WriteLine($"        Self({defaultString})");
    Console.WriteLine($"    }}");
    Console.WriteLine($"}}");
    Console.WriteLine();
}

static string GetStructName(string registerName)
{
    return string.Join("", registerName.Split('_').Select(x => x[0].ToString() + x.Substring(1).ToLowerInvariant()));
}


[XmlRoot("registerdefinition")]
public class RegisterDefinition
{
    public string DeviceName { get; set; } = default!;
    [XmlElement("Register")]
    public List<Register> Register { get; set; } = new();
}

public class Register
{
    public string Name { get; set; } = default!;
    public string Address { get; set; } = default!;
    public string? Description { get; set; }
[XmlElement("Bitfield")]
public List<Bitfield> Bitfield { get; set; } = new();
    public string RegReset { get; set; } = default!;
}

public class Bitfield
{
    public string Name { get; set; } = default!;
    public int Start { get; set; }
    public int Stop { get; set; }
    public string Access { get; set; } = default!;
    public string Reset { get; set; } = default!;
    [XmlElement("Value")]
    public List<Value> Value { get; set; } = new();
    public string? Description { get; set; }
}

public class Value
{
    public string Number { get; set; } = default!;
    public string Brief { get; set; } = default!;
}