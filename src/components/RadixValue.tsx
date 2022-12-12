import React from 'react';

interface Props {
  value: number
  format: string
  digits?: number
  caps?: boolean
}

const RadixValue = ({ value, format, digits = undefined, caps = false }: Props): JSX.Element => {
  let display;
  if (format === 'dec') {
    display = value.toString();
  } else if (format === 'bin') {
    display = value.toString(2).padStart(digits ?? 32, '0');
  } else if (format === 'hex') {
    display = value.toString(16).padStart(digits ?? 8, '0');
    if (caps) {
      display = display.toUpperCase();
    }
  } else {
    console.error(`Invalid radix format: ${format}`);
    display = value.toString();
  }

  return (
    <>{display}</>
  );
};

export default RadixValue;
