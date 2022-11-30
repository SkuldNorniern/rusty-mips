import React from 'react';
import Dropdown from 'react-bootstrap/Dropdown';

interface InnerProps {
  children: JSX.Element
  onClick: React.MouseEventHandler<HTMLSpanElement>
}

interface Props {
  value: number
  format: string
  changeFormat: (type: 'dec' | 'hex' | 'bin') => void
}

const MenuToggle = React.forwardRef<HTMLSpanElement, InnerProps>(({ children, onClick }, ref) => {
  return (
    <span ref={ref} onContextMenu={(e) => {
      e.preventDefault();
      onClick(e);
    }}>
      {children}
    </span>
  );
});

MenuToggle.displayName = 'MenuToggle';

const EditableRegisterValue = ({ value, format, changeFormat }: Props): JSX.Element => {
  let display;
  if (format === 'dec') {
    display = value.toString();
  } else if (format === 'bin') {
    display = value.toString(2).padStart(32, '0');
  } else if (format === 'hex') {
    display = value.toString(16).padStart(8, '0');
  } else {
    display = '';
  }

  const handleSelect = (key: any): void => {
    switch (key) {
      case 'edit':
        alert('todo');
        break;
      case 'viewBin':
        changeFormat('bin');
        break;
      case 'viewDec':
        changeFormat('dec');
        break;
      case 'viewHex':
        changeFormat('hex');
        break;
      default:
        break;
    }
  };

  return (
    <Dropdown onSelect={handleSelect}>
      <Dropdown.Toggle as={MenuToggle}>
        {display}
      </Dropdown.Toggle>
      <Dropdown.Menu>
        <Dropdown.Item eventKey="edit">수정</Dropdown.Item>
        <Dropdown.Divider/>
        <Dropdown.Item eventKey="viewBin">2진수로 표시</Dropdown.Item>
        <Dropdown.Item eventKey="viewDec">10진수로 표시</Dropdown.Item>
        <Dropdown.Item eventKey="viewHex">16진수로 표시</Dropdown.Item>
      </Dropdown.Menu>
    </Dropdown>
  );
};

export default EditableRegisterValue;
