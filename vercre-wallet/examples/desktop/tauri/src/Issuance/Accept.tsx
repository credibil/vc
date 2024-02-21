import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Typography from "@mui/material/Typography";
import { IssuanceView, CredentialConfiguration } from "shared_types/types/shared_types";

import VcCard, { VcCardProps } from '../Credentials/VcCard';

export type AcceptProps = {
    model: IssuanceView;
    onCancel: () => void;
    onChange: () => void;
};

export const Accept = (props: AcceptProps) => {
    const { model, onCancel, onChange } = props;

    const displayProps = (credential: CredentialConfiguration) : VcCardProps => {
        const display = credential.display?.at(0);
        return {
            backgroundColor: display?.background_color || undefined,
            color: display?.text_color || undefined,
            issuer: model.issuer,
            logo: undefined,
            logoUrl: display?.logo?.uri || undefined,
            name: display?.name,
            onSelect: undefined,
            size: 'large'
        };
    };

    return (
        <Stack spacing={2} sx={{ pt: 2 }}>
            <Typography sx={{ pb: 1, textAlign: 'center' }}>
                Do you accept the following credentials?
            </Typography>
            {Object.entries(model?.offered).map(([key, supported]) => (
                <Box key={key} sx={{ display: 'flex', justifyContent: 'center'}}>
                    <VcCard { ...displayProps(supported) } />
                </Box>
            ))}
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
                    onClick={onChange}
                    variant="contained"
                >
                    Accept
                </Button>
            </Box>
        </Stack>
    );
}

export default Accept;
