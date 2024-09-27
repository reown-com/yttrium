import Foundation

extension FFITransaction: Codable {

    private enum CodingKeys: String, CodingKey {
        case _to
        case _value
        case _data
    }
    
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        
        let to = try container.decode(String.self, forKey: ._to)
        let value = try container.decode(String.self, forKey: ._value)
        let data = try container.decode(String.self, forKey: ._data)
        
        _to = to.intoRustString()
        _value = value.intoRustString()
        _data = data.intoRustString()
    }
    
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(to, forKey: ._to)
        try container.encode(value, forKey: ._value)
        try container.encode(data, forKey: ._data)
    }
    
    public var to: String {
        _to.toString()
    }
    
    public var value: String {
        _value.toString()
    }
    
    public var data: String {
        _value.toString()
    }
    
    public init(to: String, value: String, data: String) {
        self._to = to.intoRustString()
        self._value = value.intoRustString()
        self._data = data.intoRustString()
    }
}
