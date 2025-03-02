import math

def generate_sine_lookup_table(table_size=256, scale=32767):
    """
    Generate a sine lookup table.

    Parameters:
    - table_size: The number of entries in the lookup table.
    - scale: The scaling factor for the sine values.

    Returns:
    - A list containing the sine lookup table.
    """
    sine_table = []
    for i in range(table_size):
        # Calculate the angle in radians
        angle = (i / table_size) * 2 * math.pi
        # Calculate the sine value and scale it
        sine_value = int(math.sin(angle) * scale + scale)
        sine_table.append(sine_value)
    return sine_table

# Example usage
sine_lookup_table = generate_sine_lookup_table()
print(sine_lookup_table)