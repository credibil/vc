import { ChangeEvent, useRef, useState } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import Typography from "@mui/material/Typography";

export type EnterPinProps = {
    value: string;
    onCancel: () => void;
    onChange: (offer: string) => void;
};

// TODO: This will be provided by the credential offer tx_code and should not be hard-coded
const PIN_LENGTH = 6;

export const EnterPin = (props: EnterPinProps) => {
    const { value, onCancel, onChange } = props;
    const [ pin, setPin ] = useState<Array<string>>(() => {
        const array = new Array(PIN_LENGTH).fill('');
        array.splice(0, value.length, ...value);
        return array;
    });
    const inputRef = useRef<Array<HTMLInputElement>>([]);

    const handleInputChange = (index: number) => (e: ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value.replace(/\D/g, '');

        if (val.length !== 1) {
            return;
        }
        let nextIndex = index + 1;
        if (nextIndex >= pin.length) {
            nextIndex = pin.length - 1;
        }
        inputRef.current[nextIndex].focus();
        setPin((prev) => {
            const newPin = [...prev];
            newPin[index] = val;
            return newPin;
        });
    };

    const handleEnter = () => {
        const pinValue = pin.join('');
        onChange(pinValue);
    };

    return (
        <Stack spacing={2} sx={{ pt: 2 }}>
            <Typography sx={{ pb: 1, textAlign: 'center' }}>
                Enter the user pin sent via email
            </Typography>
            <Box
                sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    gap: 2,
                }}
            >
                {pin.map((p, index) => (
                    <TextField
                        key={index}
                        inputRef={ref => inputRef.current[index] = ref}
                        inputProps={{ maxLength: 1 }}
                        onChange={handleInputChange(index)}
                        size="small"
                        value={p}
                        variant="outlined"
                        sx={{
                            maxWidth: 40,
                            textAlign: 'center',
                        }}
                    />
                ))}
            </Box>
            <Box
                sx={{
                    display: 'flex',
                    my: 2,
                    justifyContent: 'center',
                    gap: 4
                }}
            >
                <Button
                    onClick={onCancel}
                    variant="outlined"
                >
                    Cancel
                </Button>
                <Button
                    onClick={handleEnter}
                    variant="contained"
                >
                    Enter
                </Button>
            </Box>
        </Stack>
    );
}

export default EnterPin;
