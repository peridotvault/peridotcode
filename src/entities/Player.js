export class Player {
    constructor(scene, x, y) {
        this.scene = scene;
        this.sprite = scene.physics.add.sprite(x, y, 'player');
        this.sprite.setBounce(0.1);
        this.sprite.setCollideWorldBounds(true);
        
        // Movement properties
        this.speed = 200;
        this.jumpPower = 330;
        
        // Create cursor keys
        this.cursors = scene.input.keyboard.createCursorKeys();
        this.jumpKey = scene.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.SPACE);
    }

    update() {
        // Horizontal movement
        if (this.cursors.left.isDown) {
            this.sprite.setVelocityX(-this.speed);
        } else if (this.cursors.right.isDown) {
            this.sprite.setVelocityX(this.speed);
        } else {
            this.sprite.setVelocityX(0);
        }

        // Jumping
        if ((this.cursors.up.isDown || this.jumpKey.isDown) && this.sprite.body.touching.down) {
            this.sprite.setVelocityY(-this.jumpPower);
        }
    }
}