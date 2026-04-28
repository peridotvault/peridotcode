/**
 * Player entity
 * Handles player movement, physics, and interactions
 */

export default class Player {
    /**
     * @param {Phaser.Scene} scene - The scene this player belongs to
     * @param {number} x - Initial X position
     * @param {number} y - Initial Y position
     */
    constructor(scene, x, y) {
        this.scene = scene;

        // Create the player sprite
        this.sprite = scene.physics.add.sprite(x, y, 'player');

        // Configure physics properties
        this.sprite.setBounce(0.1);
        this.sprite.setCollideWorldBounds(true);

        // Movement settings
        this.speed = 200;
        this.jumpSpeed = 330;

        // Track if player is touching down (for jumping)
        this.sprite.body.setSize(32, 32);

        // Setup collision with platforms
        if (scene.platforms) {
            scene.physics.add.collider(this.sprite, scene.platforms);
        }
    }

    /**
     * Update player state based on input
     * @param {object} cursors - Phaser cursor keys object
     */
    update(cursors) {
        // Horizontal movement
        if (cursors.left.isDown) {
            this.sprite.setVelocityX(-this.speed);
        } else if (cursors.right.isDown) {
            this.sprite.setVelocityX(this.speed);
        } else {
            this.sprite.setVelocityX(0);
        }

        // Jump with space or up arrow (only when touching down)
        if ((cursors.space.isDown || cursors.up.isDown) && this.sprite.body.touching.down) {
            this.sprite.setVelocityY(-this.jumpSpeed);
        }
    }
    
    /**
     * Get player position
     * @returns {object} Position with x and y properties
     */
    getPosition() {
        return {
            x: this.sprite.x,
            y: this.sprite.y
        };
    }
    
    /**
     * Set player position
     * @param {number} x - X coordinate
     * @param {number} y - Y coordinate
     */
    setPosition(x, y) {
        this.sprite.setPosition(x, y);
    }
}
