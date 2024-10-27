import { ReactElement } from "react";
import { Grid } from "@project-gauntlet/api/components";

const nabooImage = "https://static.wikia.nocookie.net/star-wars-canon/images/2/24/NabooFull-SW.png/revision/latest/scale-to-width-down/150?cb=20151218205422"
const rylothImage = "https://static.wikia.nocookie.net/star-wars-canon/images/4/48/Dagobah_ep3.jpg/revision/latest/scale-to-width-down/150?cb=20161103221846"
const tatooineImage = "https://static.wikia.nocookie.net/star-wars-canon/images/b/b7/Ryloth_Rebels.png/revision/latest/scale-to-width-down/150?cb=20161103040944"
const dagobahImage = "https://static.wikia.nocookie.net/star-wars-canon/images/b/b0/Tatooine_TPM.png/revision/latest/scale-to-width-down/150?cb=20151124205032"
const coruscantImage = "https://static.wikia.nocookie.net/star-wars-canon/images/7/7d/Death_Star_detail.png/revision/latest/scale-to-width-down/150?cb=20151216212148"
const endorImage = "https://static.wikia.nocookie.net/star-wars-canon/images/9/96/Endor-DB.png/revision/latest/scale-to-width-down/150?cb=20160711234205"
const deathstarImage = "https://static.wikia.nocookie.net/starwars/images/a/a6/Coruscant-SWJS.jpg/revision/latest/scale-to-width-down/150?cb=20240324185443"
const dathomirImage = "https://static.wikia.nocookie.net/starwars/images/3/34/DathomirJFO.jpg/revision/latest/scale-to-width-down/150?cb=20200222032237"
const dantooineImage = "https://static.wikia.nocookie.net/starwars/images/a/a5/Dantooine_Resistance.jpg/revision/latest/scale-to-width-down/150?cb=20200120190043"

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <Grid>
            <Grid.Item title="Naboo">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: nabooImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Ryloth">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: rylothImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Tatooine">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: tatooineImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Dagobah">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: dagobahImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Coruscant">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: coruscantImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Endor">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: endorImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Death Star">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: deathstarImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Dathomir">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: dathomirImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
            <Grid.Item title="Dantooine">
                <Grid.Item.Content>
                    <Grid.Item.Content.Image source={{ url: dantooineImage }}/>
                </Grid.Item.Content>
            </Grid.Item>
        </Grid>
        // docs-code-segment:end
    )
}