import React from 'react';

interface Props {
  value: number
  format: string
}

const RadixValue = ({ value, format }: Props): JSX.Element => {
  let display;
  if (format === 'dec') {
    display = value.toString();
  } else if (format === 'bin') {
    display = value.toString(2).padStart(32, '0');
  } else if (format === 'hex') {
    display = value.toString(16).padStart(8, '0');
  } else {
    console.error(`Invalid radix format: ${format}`);
    display = value.toString();
  }

  return (
    <>{display}</>
  );
};

export default RadixValue;
